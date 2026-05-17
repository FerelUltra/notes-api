use crate::services::users::UserService;
use redis::Client;
use sqlx::PgPool;
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_service: UserService,
    pub redis: Client,
    pub jwt_secret: String,
}
