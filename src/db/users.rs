use sqlx::PgPool;

use crate::{
	dto::{CreateUserDto, UpdateUserDto},
	errors::AppError,
	models::User
};

pub async fn get_users(pool: &PgPool) -> Result<Vec<User>, AppError> {
	let users = sqlx::query_as::<_,User>(
		r#"
		select id, name
		from users
		order by id
		"#,
	)
	.fetch_all(pool)
	.await?;

	Ok(users)
}

pub async fn get_user_by_id(pool: &PgPool, id: i32) -> Result<Option<User>, AppError> {
	let user = sqlx::query_as::<_, User>(
		r#"
		select id, name
		from users
		where id = $1
		"#,
	)
	.bind(id)
	.fetch_optional(pool)
	.await?;

	Ok(user)
}

pub async fn create_user(pool: &PgPool, dto: CreateUserDto) -> Result<User, AppError> {
	let user = sqlx::query_as::<_, User>(
		r#"
		insert into users (name)
		values ($1)
		returning id, name
		"#,
	)
	.bind(dto.name)
	.fetch_one(pool)
	.await?;

	Ok(user)
}

pub async fn update_user(
	pool: &PgPool,
	id: i32,
	dto: UpdateUserDto,
) -> Result<Option<User>, AppError> {
	let user = sqlx::query_as::<_, User> (
		r#"
		update users
		set name = $1
		where id = $2
		returning id, name
		"#,
	)
	.bind(dto.name)
	.bind(id)
	.fetch_optional(pool)
	.await?;

	Ok(user)
}

pub async fn delete_user(pool: &PgPool, id: i32) -> Result<bool, AppError> {
	let result = sqlx::query(
		r#"
		delete from users
		where id = $1
		"#,
	)
	.bind(id)
	.execute(pool)
	.await?;

	Ok(result.rows_affected() > 0)
}