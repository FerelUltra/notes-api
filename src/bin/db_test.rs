use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
	dotenv().ok();

	let database_url = std::env::var("DATABASE_URL")
		.expect("DATABASE_URL not set");

	let pool = PgPoolOptions::new()
		.connect(&database_url)
		.await
		.expect("Failed to connect");

	println!("Connected!");

	let rows = sqlx::query("select id, name from users order by id")
		.fetch_all(&pool)
		.await
		.expect("Failed to fetch users");

	println!("Rows: {:?}", rows);
}