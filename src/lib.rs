use axum::{
	routing::get,
	Router
};
use tower_http::trace::TraceLayer;

pub mod db;
pub mod dto;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod state;

use handlers::users::{create_user, delete_user, get_user_by_id, get_users, update_user,};
use state::AppState;

pub fn create_app(state: AppState) -> Router {
	Router::new()
		.route("/users", get(get_users).post(create_user))
		.route("/users/{id}", get(get_user_by_id).put(update_user).delete(delete_user))
		.with_state(state)
		.layer(TraceLayer::new_for_http())
}