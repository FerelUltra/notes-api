use axum::{
    Json, Router, extract::{Path, State}, http::StatusCode, routing::{get}
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions, FromRow};
use dotenvy::dotenv;

#[derive(Serialize, Clone, FromRow)]

struct User {
    id: i32,
    name: String,
}

#[derive(Deserialize)]
struct CreateUserDto {
    name: String,
}

#[derive(Deserialize)]
struct UpdateUserDto{
    name: String,
}

#[derive(Clone)]
struct AppState{
    db: PgPool,
}


#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    let state = AppState{db};

    let app = Router::new()
        .route("/user", get(get_user))
        .route("/users", get(get_users).post(create_user))
        .route("/users/{id}", get(get_user_by_id).put(update_user).delete(delete_user))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind 127.0.0.1:3000");

    println!("Server started on http://127.0.0.1:3000");

    axum::serve(listener, app).await.expect("server failed");
}

async fn get_user() -> Json<User> {
    Json(User {
        id: 1,
        name: "Ferel".to_string(),
    })
}

async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name FROM users ORDER BY id"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(users))
}


async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>) -> Result<Json<User>, (StatusCode, String)> {

        let user = sqlx::query_as::<_, User>(
            "select id, name from users where id = $1"
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;

        match user {
            Some(user) => Ok(Json(user)),
            None => Err((StatusCode::NOT_FOUND, format!("User with id {} not found", id)))
        }
}


async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {

        let user = sqlx::query_as::<_, User>(
            "insert into users (name)
            values ($1)
            returning id, name"
        )
        .bind(payload.name)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;

        Ok((StatusCode::CREATED, Json(user)))
}

async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>(
        "update users
        set name = $1
        where id = $2
        returning id, name"
    )
    .bind(payload.name)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;

    match user{
        Some(user) => Ok(Json(user)),
        None => Err((StatusCode::NOT_FOUND, format!("User with id {} not found", id)))
    }
    
}

async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("delete from users where id = $1")
    .bind(id)
    .execute(&state.db)
    .await.map_err(internal_error)?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, format!("User with id {} not found", id)))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}