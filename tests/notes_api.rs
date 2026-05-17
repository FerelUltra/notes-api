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
        eprintln!("skipping notes API test: TEST_DATABASE_URL is not set");
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

async fn create_note(app: axum::Router, token: &str, title: &str, content: &str) -> Value {
    let body = format!(r#"{{"title":"{}","content":"{}"}}"#, title, content);

    let request = Request::builder()
        .uri("/notes")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn create_note_should_return_201_and_created_note() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;

    let note = create_note(app, &token, "First note", "Hello").await;

    assert_eq!(note["id"], 1);
    assert_eq!(note["title"], "First note");
    assert_eq!(note["content"], "Hello");
}

#[tokio::test]
async fn get_notes_should_return_only_authenticated_users_notes() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let alice_token = register_user(app.clone(), "Alice", "alice@example.com").await;
    let bob_token = register_user(app.clone(), "Bob", "bob@example.com").await;

    create_note(app.clone(), &alice_token, "Alice note", "Private").await;
    create_note(app.clone(), &bob_token, "Bob note", "Also private").await;

    let request = Request::builder()
        .uri("/notes")
        .method("GET")
        .header("authorization", format!("Bearer {}", alice_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let notes: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(notes.as_array().unwrap().len(), 1);
    assert_eq!(notes[0]["title"], "Alice note");
}

#[tokio::test]
async fn get_note_by_id_should_return_own_note() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;
    create_note(app.clone(), &token, "Readable", "Content").await;

    let request = Request::builder()
        .uri("/notes/1")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let note: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(note["id"], 1);
    assert_eq!(note["title"], "Readable");
}

#[tokio::test]
async fn update_note_should_update_own_note() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;
    create_note(app.clone(), &token, "Old title", "Old content").await;

    let request = Request::builder()
        .uri("/notes/1")
        .method("PUT")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            r#"{"title":"New title","content":"New content"}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let note: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(note["title"], "New title");
    assert_eq!(note["content"], "New content");
}

#[tokio::test]
async fn delete_note_should_delete_own_note() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;
    create_note(app.clone(), &token, "Delete me", "Content").await;

    let request = Request::builder()
        .uri("/notes/1")
        .method("DELETE")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let request = Request::builder()
        .uri("/notes/1")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn notes_routes_should_return_401_without_token() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let request = Request::builder()
        .uri("/notes")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_note_should_return_400_when_title_is_empty() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let token = register_user(app.clone(), "Alice", "alice@example.com").await;

    let request = Request::builder()
        .uri("/notes")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(r#"{"title":"   ","content":"Content"}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn user_should_not_access_another_users_note() {
    let Some(app) = setup_test_app().await else {
        return;
    };

    let alice_token = register_user(app.clone(), "Alice", "alice@example.com").await;
    let bob_token = register_user(app.clone(), "Bob", "bob@example.com").await;
    create_note(app.clone(), &alice_token, "Alice note", "Private").await;

    let get_request = Request::builder()
        .uri("/notes/1")
        .method("GET")
        .header("authorization", format!("Bearer {}", bob_token))
        .body(Body::empty())
        .unwrap();

    let get_response = app.clone().oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    let update_request = Request::builder()
        .uri("/notes/1")
        .method("PUT")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", bob_token))
        .body(Body::from(r#"{"title":"Stolen","content":"Nope"}"#))
        .unwrap();

    let update_response = app.clone().oneshot(update_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::NOT_FOUND);

    let delete_request = Request::builder()
        .uri("/notes/1")
        .method("DELETE")
        .header("authorization", format!("Bearer {}", bob_token))
        .body(Body::empty())
        .unwrap();

    let delete_response = app.oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::NOT_FOUND);
}
