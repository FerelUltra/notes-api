use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

pub mod auth;
pub mod db;
pub mod dto;
pub mod errors;
pub mod handlers;
pub mod jwt;
pub mod models;
pub mod services;
pub mod state;

use handlers::{
    auth::{login, register},
    notes::get_notes,
    users::{delete_user, get_user_by_id, get_users, update_user},
};

use state::AppState;

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/users", get(get_users))
        .route(
            "/users/{id}",
            get(get_user_by_id).put(update_user).delete(delete_user),
        )
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/notes", get(get_notes))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
