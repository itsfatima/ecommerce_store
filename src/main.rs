use actix_web::{web, App, HttpResponse, HttpServer, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::error::Error;
use dotenv::dotenv;
use std::env;



#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct Product {
    id: i32,
    name: String,
    description: Option<String>,
    price: f64,
}

async fn get_products(pool: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    let result = sqlx::query_as::<_, Product>("SELECT id, name, description, price FROM products")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(result))
}

async fn create_product(
    pool: web::Data<PgPool>,
    product: web::Json<Product>,
) -> Result<HttpResponse, actix_web::Error> {
    let new_product = product.into_inner();
    sqlx::query(
        "INSERT INTO products (name, description, price) VALUES ($1, $2, $3)",
    )
    .bind(&new_product.name)
    .bind(&new_product.description)
    .bind(new_product.price)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Created().finish())
}

pub async fn init_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("postgres://postgres:03075@localhost:5432/postgres").expect("DATABASE_URL is not set");
    PgPool::connect(&database_url).await
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/products", web::get().to(get_products))
            .route("/products", web::post().to(create_product))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}