use serde::{Serialize, Deserialize, Serializer, Deserializer};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use sqlx::postgres::PgRow;
use sqlx::Row;

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

pub struct OrderTracking {
    pub id: i32,
    pub order_id: i32,
    pub status: String,
    pub location: Option<String>,
    pub timestamp: chrono::NaiveDateTime,
}

impl OrderTracking {
    pub fn from_row(row: PgRow) -> Self {
        let id = row.get("id");
        let order_id = row.get("order_id");
        let status = row.get("status");
        let location = row.get("location");
        let timestamp = row.get("timestamp");

        Self {
            id,
            order_id,
            status,
            location,
            timestamp,
        }
    }
}

// Define a newtype wrapper for NaiveDateTime
#[derive(Serialize, Deserialize)]
pub struct SerializableNaiveDateTime(pub NaiveDateTime);

// Implement Serialize for SerializableNaiveDateTime
impl Serialize for SerializableNaiveDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted = self.0.to_string();
        serializer.serialize_str(&formatted)
    }
}

// Implement Deserialize for SerializableNaiveDateTime
impl<'de> Deserialize<'de> for SerializableNaiveDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let naive_datetime = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
            .map_err(serde::de::Error::custom)?;
        Ok(SerializableNaiveDateTime(naive_datetime))
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Checkout {
    pub id: i32,
    pub user_id: i32,
    pub total_price: f64,
    pub discount_amount: f64,
    pub final_price: f64,
    pub order_id: i32,
    pub created_at: SerializableNaiveDateTime, // Wrap created_at with SerializableNaiveDateTime
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DiscountCoupon {
    pub id: i32,
    pub code: String,
    pub discount_amount: BigDecimal,
    pub expiration_date: NaiveDate,
}

