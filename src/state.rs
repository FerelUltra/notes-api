use sqlx::PgPool;
use crate::services::users::UserService;
#[derive(Clone)]
pub struct AppState {
	pub db: PgPool,
	pub user_service: UserService
}