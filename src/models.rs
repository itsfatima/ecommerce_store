use sqlx::FromRow;
use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;


#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub price: Decimal,
}

#[derive(sqlx::FromRow)]
pub struct Order {
    pub id: i32,
    pub user_id: i32,
    pub total_price: String, // Use String for total_price.
    pub status: String,
}

#[derive(sqlx::FromRow)]
#[derive(Debug)]
pub struct CartItem {
    pub id: i32,
    pub user_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub price: Decimal, // Use Decimal instead of BigDecimal.
}

#[derive(sqlx::FromRow)]
pub struct OrderTracking {
    pub id: i32,
    pub order_id: i32,
    pub status: String,
    pub location: String,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(sqlx::FromRow)]
pub struct DiscountCoupon {
    pub id: i32,
    pub code: String,
    pub discount_amount: f64,
    pub expiration_date: chrono::NaiveDate,
}

