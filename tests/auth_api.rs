use axum::{
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use notes_api::{create_app, services::users::UserService, state::AppState};
use redis::AsyncCommands;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower::ServiceExt;

async fn setup_test_app() -> Option<axum::Router> {
    dotenvy::dotenv().ok();

    let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipping auth API test: TEST_DATABASE_URL is not set");
        return None;
    };

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("failed to connect to test database");

    sqlx::query("TRUNCATE TABLE notes, users RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await
        .expect("failed to clean test tables");

    let user_service = UserService::new(pool.clone());
    let redis =
        redis::Client::open("redis://127.0.0.1:6379").expect("failed to create redis client");

    if let Ok(mut conn) = redis.get_multiplexed_async_connection().await {
        let _: redis::RedisResult<()> = conn
            .del(&["rate_limit:register:127.0.0.1", "users:all"])
            .await;
    }

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let state = AppState {
        db: pool,
        user_service,
        redis,
        jwt_secret,
    };

    Some(create_app(state).layer(MockConnectInfo(SocketAddr::from(([127, 0, 0, 1], 3000)))))
}

async fn register_user(app: axum::Router) -> Value {
    let request = Request::builder()
        .uri("/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"name":"Alice","email":"alice@example.com","password":"secret123"}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

async fn login_user(app: axum::Router) -> Value {
    let request = Request::builder()
        .uri("/auth/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"alice@example.com","password":"secret123"}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

async fn refresh_token(app: axum::Router, refresh_token: &str) -> (StatusCode, Value) {
    let body = serde_json::json!({
        "refresh_token": refresh_token,
    });

    let request = Request::builder()
        .uri("/auth/refresh")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    if body.is_empty() {
        return (status, Value::Null);
    }

    (status, serde_json::from_slice(&body).unwrap())
}

async fn logout(app: axum::Router, refresh_token: &str) -> StatusCode {
    let body = serde_json::json!({
        "refresh_token": refresh_token,
    });

    let request = Request::builder()
        .uri("/auth/logout")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    app.oneshot(request).await.unwrap().status()
}

#[tokio::test]
async fn register_should_return_access_and_refresh_tokens() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let body = register_user(app).await;

    assert!(body["access_token"].as_str().is_some());
    assert!(body["refresh_token"].as_str().is_some());
    assert_eq!(body["token"], "Bearer");
}

#[tokio::test]
async fn login_should_return_access_and_refresh_tokens() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    register_user(app.clone()).await;

    let body = login_user(app).await;

    assert!(body["access_token"].as_str().is_some());
    assert!(body["refresh_token"].as_str().is_some());
    assert_eq!(body["token"], "Bearer");
}

#[tokio::test]
async fn refresh_should_rotate_refresh_token() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let register_body = register_user(app.clone()).await;
    let first_refresh_token = register_body["refresh_token"].as_str().unwrap();

    let (status, refresh_body) = refresh_token(app.clone(), first_refresh_token).await;

    assert_eq!(status, StatusCode::OK);
    assert!(refresh_body["access_token"].as_str().is_some());
    assert!(refresh_body["refresh_token"].as_str().is_some());
    assert_ne!(
        first_refresh_token,
        refresh_body["refresh_token"].as_str().unwrap()
    );

    let (status, _) = refresh_token(app, first_refresh_token).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_should_revoke_refresh_token() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let register_body = register_user(app.clone()).await;
    let refresh_token_value = register_body["refresh_token"].as_str().unwrap();

    let status = logout(app.clone(), refresh_token_value).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = refresh_token(app, refresh_token_value).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
