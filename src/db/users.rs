use sqlx::PgPool;

use crate::{dto::users::UpdateUserDto, errors::AppError, models::users::User};

pub struct CreateUserDb {
    pub name: String,
    pub email: String,
    pub password_hash: String,
}

pub async fn get_users(pool: &PgPool) -> Result<Vec<User>, AppError> {
    let users = sqlx::query_as::<_, User>(
        r#"
		select id, name, email, password_hash, token_version
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
		select id, name, email, password_hash, token_version
		from users
		where id = $1
		"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_name(
    pool: &PgPool,
    name: &str,
) -> Result<Option<(User, String)>, AppError> {
    let record = sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, password_hash, token_version
        FROM users
        WHERE name = $1
        "#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    let result = record
        .map(|r| {
            let password_hash = r.password_hash.ok_or_else(|| {
                AppError::InternalServerError("User has no password hash".to_string())
            })?;
            Ok::<(User, String), AppError>((
                User {
                    id: r.id,
                    name: r.name,
                    email: r.email,
                    password_hash: None,
                    token_version: r.token_version
                },
                password_hash,
            ))
        })
        .transpose()?;

    Ok(result)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, password_hash, token_version
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email.trim().to_lowercase())
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn create_user(pool: &PgPool, dto: CreateUserDb) -> Result<User, AppError> {
    let name = dto.name.trim().to_string();
    let email = dto.email.trim().to_lowercase();

    let user = sqlx::query_as::<_, User>(
        r#"
		insert into users (name, email, password_hash)
		values ($1, $2, $3)
		returning id, name, email, password_hash, token_version
		"#,
    )
    .bind(name)
    .bind(email)
    .bind(dto.password_hash)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn update_user(
    pool: &PgPool,
    id: i32,
    dto: UpdateUserDto,
) -> Result<Option<User>, AppError> {
    let name = dto.name.trim().to_string();

    let user = sqlx::query_as::<_, User>(
        r#"
		update users
		set name = $1
		where id = $2
		returning id, name, email, password_hash, token_version
		"#,
    )
    .bind(name)
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

pub async fn get_user_token_version(pool: &PgPool, user_id: i32) -> Result<Option<i32>, AppError> {
    let token_version = sqlx::query_scalar::<_, i32> (
        r#"
        SELECT token_version
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(token_version)
}

pub async fn increment_token_version(pool: &PgPool, user_id: i32) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE users
        SET token_version = token_version + 1
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_db() -> Option<PgPool> {
        dotenvy::dotenv().ok();

        let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
            eprintln!("skipping DB test: TEST_DATABASE_URL is not set");
            return None;
        };

        let pool = PgPoolOptions::new()
            .connect(&database_url)
            .await
            .expect("failed to clean users table");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");

        Some(pool)
    }

    async fn clean_users_table(pool: &PgPool) {
        sqlx::query("truncate table notes, users restart identity cascade")
            .execute(pool)
            .await
            .expect("failed to clean users table");
    }

    #[tokio::test]
    async fn create_user_should_insert_user_into_db() {
        let Some(pool) = setup_test_db().await else {
            return;
        };
        clean_users_table(&pool).await;

        let user = create_user(
            &pool,
            CreateUserDb {
                name: "Ferel".to_string(),
                email: "ferel@example.com".to_string(),
                password_hash: "password-hash".to_string(),
            },
        )
        .await
        .expect("create_user failed");

        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Ferel");
        assert_eq!(user.email.as_deref(), Some("ferel@example.com"));
    }

    #[tokio::test]
    async fn get_user_by_id_should_return_user_when_user_exists() {
        let Some(pool) = setup_test_db().await else {
            return;
        };
        clean_users_table(&pool).await;

        let created = create_user(
            &pool,
            CreateUserDb {
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                password_hash: "password-hash".to_string(),
            },
        )
        .await
        .expect("create_user failed");

        let user = get_user_by_id(&pool, created.id)
            .await
            .expect("get_user_by_id failed");

        assert!(user.is_some());

        let user = user.unwrap();
        assert_eq!(user.id, created.id);
        assert_eq!(user.name, "Alice");
    }

    #[tokio::test]
    async fn get_user_by_id_should_return_none_when_user_does_not_exist() {
        let Some(pool) = setup_test_db().await else {
            return;
        };
        clean_users_table(&pool).await;

        let user = get_user_by_id(&pool, 999)
            .await
            .expect("get_user_by_id failed");

        assert!(user.is_none());
    }

    #[tokio::test]
    async fn get_users_should_return_all_users() {
        let Some(pool) = setup_test_db().await else {
            return;
        };
        clean_users_table(&pool).await;

        create_user(
            &pool,
            CreateUserDb {
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                password_hash: "password-hash".to_string(),
            },
        )
        .await
        .expect("create_user failed");

        create_user(
            &pool,
            CreateUserDb {
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                password_hash: "password-hash".to_string(),
            },
        )
        .await
        .expect("create_user failed");

        let users = get_users(&pool).await.expect("get_users failed");

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "Alice");
        assert_eq!(users[1].name, "Bob");
    }

    #[tokio::test]
    async fn update_user_should_update_existing_user() {
        let Some(pool) = setup_test_db().await else {
            return;
        };
        clean_users_table(&pool).await;

        let created = create_user(
            &pool,
            CreateUserDb {
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                password_hash: "password-hash".to_string(),
            },
        )
        .await
        .expect("create_user failed");

        let updated = update_user(
            &pool,
            created.id,
            UpdateUserDto {
                name: "Alice updated".to_string(),
            },
        )
        .await
        .expect("update_user failed");
        assert!(updated.is_some());

        let updated = updated.unwrap();
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "Alice updated")
    }

    #[tokio::test]
    async fn update_user_should_return_none_when_user_does_not_exist() {
        let Some(pool) = setup_test_db().await else {
            return;
        };
        clean_users_table(&pool).await;

        let updated = update_user(
            &pool,
            999,
            UpdateUserDto {
                name: "Nobody".to_string(),
            },
        )
        .await
        .expect("update_user failed");

        assert!(updated.is_none());
    }

    #[tokio::test]
    async fn delete_user_should_delete_existing_user() {
        let Some(pool) = setup_test_db().await else {
            return;
        };
        clean_users_table(&pool).await;

        let created = create_user(
            &pool,
            CreateUserDb {
                name: "Delete me".to_string(),
                email: "delete-me@example.com".to_string(),
                password_hash: "password-hash".to_string(),
            },
        )
        .await
        .expect("create_user failed");

        let deleted = delete_user(&pool, created.id)
            .await
            .expect("get_user_by_id failed");

        assert!(deleted);

        let user = get_user_by_id(&pool, created.id)
            .await
            .expect("get_user_by_id failed");

        assert!(user.is_none());
    }

    #[tokio::test]
    async fn delete_user_should_return_false_when_user_does_not_exist() {
        let Some(pool) = setup_test_db().await else {
            return;
        };
        clean_users_table(&pool).await;

        let deleted = delete_user(&pool, 999).await.expect("delete_user failed");

        assert!(!deleted)
    }
}
