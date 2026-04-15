use axum::{
    routing::get,
    Router,
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

mod state;
mod models;
mod dto;
mod handlers;
mod errors;
mod db;

use handlers::users::*;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let db = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    let state = AppState { db };

    let app = Router::new()
        .route("/users", get(get_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user_by_id)
                .put(update_user)
                .delete(delete_user),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}