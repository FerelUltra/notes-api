use axum::{
	body::Body,
	http::{Request, StatusCode}
};
use http_body_util::BodyExt;
use notes_api::{create_app, state::AppState};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

async fn setup_test_app() -> axum::Router {
	dotenvy::dotenv().ok();

	let database_url = std::env::var("TEST_DATABASE_URL")
		.expect("TEST_DATABASE_URL must be set");

	let pool = PgPoolOptions::new()
		.connect(&database_url)
		.await
		.expect("failed to connect to test database");

	sqlx::query("TRUNCATE TABLE users RESTART IDENTITY")
		.execute(&pool)
		.await
		.expect("failed to clean users table");

	let state = AppState {db: pool};

	create_app(state)
}

#[tokio::test]
async fn get_users_should_return_200(){
	let app = setup_test_app().await;

	let request = Request::builder()
		.uri("/users")
		.method("GET")
		.body(Body::empty())
		.unwrap();

	let response = app.oneshot(request).await.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn create_user_should_return_200_and_created_user(){
	let app = setup_test_app().await;

	let request = Request::builder()
		.uri("/users")
		.method("POST")
		.header("content-type", "application/json")
		.body(Body::from(r#"{"name":"Ferel"}"#))
		.unwrap();

	let response = app.oneshot(request).await.unwrap();

	assert_eq!(response.status(), StatusCode::OK);

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body_text = String::from_utf8(body.to_vec()).unwrap();

	assert!(body_text.contains(r#""name":"Ferel""#));
}

#[tokio::test]
async fn create_user_should_return_400_when_name_is_empty(){
	let app = setup_test_app().await;

	let request = Request::builder()
		.uri("/users")
		.method("POST")
		.header("content-type", "application/json")
		.body(Body::from(r#"{"name":"   "}"#))
		.unwrap();
	
	let response = app.oneshot(request).await.unwrap();

	assert_eq!(response.status(), StatusCode::BAD_REQUEST);

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body_text = String::from_utf8(body.to_vec()).unwrap();

	assert!(body_text.contains("Name cannot be empty"));
}

#[tokio::test]
async fn get_user_by_id_should_return_200_when_user_exists(){
	let app = setup_test_app().await;

	let create_request = Request::builder()
		.uri("/users")
		.method("POST")
		.header("content-type", "application/json")
		.body(Body::from(r#"{"name":"Alice"}"#))
		.unwrap();

	let create_response = app.clone().oneshot(create_request).await.unwrap();
	assert_eq!(create_response.status(), StatusCode::OK);

	let request = Request::builder()
		.uri("/users/1")
		.method("GET")
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
async fn get_user_by_id_should_return_404_when_user_does_not_exist(){
	let app = setup_test_app().await;

	let request = Request::builder()
		.uri("/users/999")
		.method("GET")
		.body(Body::empty())
		.unwrap();

	let response = app.oneshot(request).await.unwrap();

	assert_eq!(response.status(), StatusCode::NOT_FOUND);
}