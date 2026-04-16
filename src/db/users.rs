use sqlx::PgPool;

use crate::{
    dto::{CreateUserDto, UpdateUserDto},
    errors::AppError,
    models::User,
};

pub async fn get_users(pool: &PgPool) -> Result<Vec<User>, AppError> {
    let users = sqlx::query_as::<_, User>(
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
    let name = dto.name.trim().to_string();

    let user = sqlx::query_as::<_, User>(
        r#"
		insert into users (name)
		values ($1)
		returning id, name
		"#,
    )
    .bind(name)
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
		returning id, name
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_db() -> PgPool {
        dotenvy::dotenv().ok();

        let database_url =
            std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");

        let pool = PgPoolOptions::new()
            .connect(&database_url)
            .await
            .expect("failed to clean users table");

        sqlx::query(
            r#"
                create table if not exists users(
                id serial primary key,
                name text not null
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("failed to create users table");

        pool
    }

    async fn clean_users_table(pool: &PgPool) {
        sqlx::query("truncate table users restart identity")
            .execute(pool)
            .await
            .expect("failed to clean users table");
    }

    #[tokio::test]
    async fn create_user_should_insert_user_into_db() {
        let pool = setup_test_db().await;
        clean_users_table(&pool).await;

        let dto = CreateUserDto {
            name: "Ferel".to_string(),
        };

        let user = create_user(&pool, dto).await.expect("create_user failed");

        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Ferel");
    }

    #[tokio::test]
    async fn get_user_by_id_should_return_user_when_user_exists() {
        let pool = setup_test_db().await;
        clean_users_table(&pool).await;

        let created = create_user(
            &pool,
            CreateUserDto {
                name: "Alice".to_string(),
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
        let pool = setup_test_db().await;
        clean_users_table(&pool).await;

        let user = get_user_by_id(&pool, 999)
            .await
            .expect("get_user_by_id failed");

        assert!(user.is_none());
    }

    #[tokio::test]
    async fn get_users_should_return_all_users() {
        let pool = setup_test_db().await;
        clean_users_table(&pool).await;

        create_user(
            &pool,
            CreateUserDto {
                name: "Alice".to_string(),
            },
        )
        .await
        .expect("create_user failed");

        create_user(
            &pool,
            CreateUserDto {
                name: "Bob".to_string(),
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
        let pool = setup_test_db().await;
        clean_users_table(&pool).await;

        let created = create_user(
            &pool,
            CreateUserDto {
                name: "Alice".to_string(),
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
        let pool = setup_test_db().await;
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
        let pool = setup_test_db().await;
        clean_users_table(&pool).await;

        let created = create_user(
            &pool,
            CreateUserDto {
                name: "Delete me".to_string(),
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
        let pool = setup_test_db().await;
        clean_users_table(&pool).await;

        let deleted = delete_user(&pool, 999).await.expect("delete_user failed");

        assert!(!deleted)
    }
}
