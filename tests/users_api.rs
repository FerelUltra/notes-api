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
        eprintln!("skipping API test: TEST_DATABASE_URL is not set");
        return None;
    };

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("failed to connect to test database");

    sqlx::query("TRUNCATE TABLE notes, users RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await
        .expect("failed to clean users table");

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

async fn register_user(app: axum::Router, name: &str, email: &str) -> String {
    let body = format!(
        r#"{{"name":"{}","email":"{}","password":"secret123"}}"#,
        name, email
    );

    let request = Request::builder()
        .uri("/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body).unwrap();

    body_json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn get_users_should_return_200() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;
    register_user(app.clone(), "Bob", "bob@example.com").await;

    let request = Request::builder()
        .uri("/users")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_text = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_text.contains("\"name\":\"Alice\""));
    assert!(body_text.contains("\"name\":\"Bob\""));
}

#[tokio::test]
async fn register_user_should_return_201_and_access_token() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let request = Request::builder()
        .uri("/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"name":"Ferel","email":"ferel@example.com","password":"secret123"}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_text = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_text.contains(r#""access_token""#));
    assert!(body_text.contains(r#""token":"Bearer""#));
}

#[tokio::test]
async fn register_user_should_return_400_when_name_is_empty() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let request = Request::builder()
        .uri("/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"name":"   ","email":"ferel@example.com","password":"secret123"}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_text = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_text.contains("Name cannot be empty"));
}

#[tokio::test]
async fn get_user_by_id_should_return_200_when_user_exists() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;

    let request = Request::builder()
        .uri("/users/1")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_text = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_text.contains(r#""id":1"#));
    assert!(body_text.contains(r#""name":"Alice""#));
}

#[tokio::test]
async fn get_user_by_id_should_return_404_when_user_does_not_exist() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;

    let request = Request::builder()
        .uri("/users/999")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_user_should_return_200_and_updated_user() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;

    let updated_request = Request::builder()
        .uri("/users/1")
        .method("PUT")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from("{\"name\":\"Alice updated\"}"))
        .unwrap();

    let update_response = app.clone().oneshot(updated_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);

    let body = update_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let body_text = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_text.contains("\"id\":1"));
    assert!(body_text.contains("\"name\":\"Alice updated\""));

    let get_request = Request::builder()
        .uri("/users/1")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let get_body = get_response.into_body().collect().await.unwrap().to_bytes();
    let get_body_text = String::from_utf8(get_body.to_vec()).unwrap();

    assert!(get_body_text.contains("\"name\":\"Alice updated\""))
}

#[tokio::test]
async fn update_user_should_return_403_when_user_updates_another_user() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;
    register_user(app.clone(), "Bob", "bob@example.com").await;

    let updated_request = Request::builder()
        .uri("/users/2")
        .method("PUT")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from("{\"name\":\"Bob updated\"}"))
        .unwrap();

    let updated_response = app.clone().oneshot(updated_request).await.unwrap();

    assert_eq!(updated_response.status(), StatusCode::FORBIDDEN)
}

#[tokio::test]
async fn update_user_should_return_401_when_own_user_no_longer_exists() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;

    let delete_request = Request::builder()
        .uri("/users/1")
        .method("DELETE")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);

    let updated_request = Request::builder()
        .uri("/users/1")
        .method("PUT")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from("{\"name\":\"Nobody\"}"))
        .unwrap();

    let updated_response = app.clone().oneshot(updated_request).await.unwrap();

    assert_eq!(updated_response.status(), StatusCode::UNAUTHORIZED)
}

#[tokio::test]
async fn delete_user_should_return_200_and_remove_user() {
    let Some(app) = setup_test_app().await else {
        return;
    };
    let token = register_user(app.clone(), "Delete me", "delete-me@example.com").await;

    let delete_request = Request::builder()
        .uri("/users/1")
        .method("DELETE")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);

    let get_request = Request::builder()
        .uri("/users/1")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let get_response = app.clone().oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_user_should_return_403_when_user_deletes_another_user() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;
    register_user(app.clone(), "Bob", "bob@example.com").await;

    let delete_request = Request::builder()
        .uri("/users/2")
        .method("DELETE")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();

    assert_eq!(delete_response.status(), StatusCode::FORBIDDEN)
}

#[tokio::test]
async fn delete_user_should_return_401_when_own_user_no_longer_exists() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;

    let delete_request = Request::builder()
        .uri("/users/1")
        .method("DELETE")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);

    let delete_request = Request::builder()
        .uri("/users/1")
        .method("DELETE")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::UNAUTHORIZED)
}
