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
    auth::{login, logout, refresh, register},
    health::health,
    notes::{create_note, delete_note, get_note_by_id, get_notes, update_note},
    users::{delete_user, get_user_by_id, get_users, update_user},
};

use state::AppState;

use crate::handlers::auth::logout_all;

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/users", get(get_users))
        .route(
            "/users/{id}",
            get(get_user_by_id).put(update_user).delete(delete_user),
        )
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/notes", get(get_notes).post(create_note))
        .route(
            "/notes/{id}",
            get(get_note_by_id).put(update_note).delete(delete_note),
        )
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/auth/logout-all", post(logout_all))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
