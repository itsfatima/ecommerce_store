use chrono::{NaiveDateTime, NaiveDate};
use serde::{Serialize, Deserialize};
use sqlx::{self};
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

pub struct OrderTracking {
    pub id: i32,
    pub order_id: i32,
    pub status: String,
    pub location: Option<String>,
    pub timestamp: chrono::NaiveDateTime,
}

impl OrderTracking {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_status(&self) -> &str {
        &self.status
    }

    pub fn get_location(&self) -> &Option<String> {
        &self.location
    }

    pub fn get_timestamp(&self) -> &NaiveDateTime {
        &self.timestamp
    }
}

// // Define a newtype wrapper for NaiveDateTime
// #[derive(Debug, Serialize, Deserialize)]
// pub struct SerializableNaiveDateTime(pub NaiveDateTime);

// // Implement custom serialization and deserialization for SerializableNaiveDateTime
// mod naive_datetime_serializer {
//     use super::SerializableNaiveDateTime;
//     use chrono::NaiveDateTime;
//     use serde::{Serializer, Deserializer, Deserialize};

//     pub fn serialize<S>(date: &SerializableNaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let formatted = date.0.to_string();
//         serializer.serialize_str(&formatted)
//     }

//     pub fn deserialize<'de, D>(deserializer: D) -> Result<SerializableNaiveDateTime, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s: String = Deserialize::deserialize(deserializer)?;
//         let naive_datetime = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
//             .map_err(serde::de::Error::custom)?;
//         Ok(SerializableNaiveDateTime(naive_datetime))
//     }
// }

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Checkout {
    pub id: i32,
    pub user_id: i32,
    pub total_price: f64,
    pub discount_amount: f32,
    pub final_price: f64,
    pub order_id: i32,
    // #[serde(with = "naive_datetime_serializer")]
    // pub created_at: SerializableNaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DiscountCoupon {
    pub id: i32,
    pub code: String,
    pub discount_amount: f32,
    pub expiration_date: NaiveDate,
}