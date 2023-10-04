use actix_web::{App, HttpServer};
use sqlx::postgres::PgPool;
use std::env;

mod handlers;
mod models;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let database_url = env::var("postgres://postgres:03075@localhost:5432/postgres").expect("DATABASE_URL not set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to the database");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .configure(handlers::config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}