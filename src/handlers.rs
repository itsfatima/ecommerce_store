use actix_web::{web, HttpResponse, Responder};
use sqlx::postgres::PgRow;
use sqlx::{Row, PgPool};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use crate::models::{Product, CartItem, Order, OrderTracking, Checkout, DiscountCoupon};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(web::resource("/products").route(web::get().to(get_products)))
            .service(web::resource("/cart").route(web::get().to(get_cart)))
            .service(web::resource("/orders").route(web::get().to(get_orders)))
            .service(web::resource("/order/{order_id}").route(web::get().to(track_order)))
            .service(web::resource("/checkout").route(web::post().to(process_checkout)))
            .service(web::resource("/coupons").route(web::get().to(get_coupons))),
    );
}

async fn get_products(pool: web::Data<sqlx::PgPool>) -> impl Responder {
    // Retrieve products from the database and categorize them
    let query = sqlx::query_as::<_, Product>("SELECT * FROM products ORDER BY category, name")
        .fetch_all(pool.as_ref())
        .await;

    match query {
        Ok(products) => {
            // Create a data structure to categorize products by category
            let mut categorized_products = HashMap::<String, Vec<Product>>::new();

            for product in products {
                categorized_products
                    .entry(product.category.clone())
                    .or_insert_with(Vec::new)
                    .push(product);
            }

            // Return categorized products as JSON
            HttpResponse::Ok().json(categorized_products)
        }
        Err(_) => HttpResponse::InternalServerError().finish(), // Handle errors appropriately
    }
}

async fn get_cart(pool: web::Data<PgPool>) -> impl Responder {
    // Replace `user_id` with the actual user ID you want to retrieve the cart for.
    let user_id = 1; // Example user ID.

    // Fetch the user's cart items from the database using SQLx.
    let result = sqlx::query(
        "SELECT id, user_id, product_id, quantity, price FROM cart_items WHERE user_id = $1",
    )
    .bind(user_id)
    .map(|row: PgRow| {
        let id: i32 = row.get(0);
        let user_id: i32 = row.get(1);
        let product_id: i32 = row.get(2);
        let quantity: i32 = row.get(3);
        let price: Decimal = row.get(4); // Use Decimal here.

        Ok(CartItem {
            id,
            user_id,
            product_id,
            quantity,
            price,
        })
    })
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(cart_items) => {
            // Process cart items as needed.
            // You can calculate the total price, apply discounts, and format the response.
            let total_price = calculate_total_price(&cart_items);
            let response = format!(
                "Shopping cart contents: {:?}\nTotal Price: {}",
                cart_items, total_price
            );
            HttpResponse::Ok().body(response)
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch cart items"),
    }
}

async fn get_orders(pool: web::Data<PgPool>) -> impl Responder {
    let user_id = 1; // Example user ID.

    // Fetch the user's order history from the database using SQLx.
    let result = sqlx::query(
        "SELECT id, user_id, total_price::numeric, status FROM orders WHERE user_id = $1",
    )
    .bind(user_id)
    .map(|row: PgRow| {
        let id: i32 = row.get(0);
        let user_id: i32 = row.get(1);
        let total_price: Decimal = row.get(2); // Convert to Decimal.
        let status: String = row.get(3);

        // Convert the Decimal total_price to a formatted string.
        let total_price_str = total_price.to_string();

        Ok(Order {
            id,
            user_id,
            total_price: total_price_str,
            status,
        })
    })
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(orders) => {
            // Process and format the user's order history as needed.
            let formatted_orders: Vec<String> = orders
                .iter()
                .map(|order_result| match order_result {
                    Ok(order) => {
                        format!(
                            "Order ID: {}, Total Price: {}, Status: {}",
                            order.id, order.total_price, order.status
                        )
                    }
                    Err(_) => "Error fetching order".to_string(),
                })
                .collect();

            HttpResponse::Ok().body(formatted_orders.join("\n"))
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch orders"),
    }
}

async fn track_order(web::Path(order_id): web::Path<i32>, pool: web::Data<PgPool>) -> impl Responder {
    // Fetch order tracking information from the database using SQLx.
    let result = sqlx::query!(
        r#"
        SELECT id, order_id, status, location, timestamp
        FROM order_tracking
        WHERE order_id = $1
        "#,
        order_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let tracking_info: Vec<OrderTracking> = rows
                .into_iter()
                .map(|row| OrderTracking {
                    id: row.id,
                    order_id: row.order_id,
                    status: row.status,
                    location: row.location,
                    timestamp: row.timestamp,
                })
                .collect();

            // Process and format the order tracking information as needed.
            let formatted_info: Vec<String> = tracking_info
                .iter()
                .map(|info| format!(
                    "Tracking ID: {}, Status: {}, Location: {}, Timestamp: {}",
                    info.id, info.status, info.location, info.timestamp
                ))
                .collect();

            HttpResponse::Ok().body(formatted_info.join("\n"))
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch order tracking information"),
    }
}

async fn process_checkout(
    pool: web::Data<PgPool>,
    user_id: web::Path<i32>,
    coupon_code: web::Path<String>,
) -> Result<HttpResponse, HttpResponse> {
    // 1. Fetch the cart items for the user from the database.
    let user_id = user_id.into_inner(); // Extract the i32 value from the web::Path.
    let cart_items = fetch_cart_items_from_db(pool.get_ref(), user_id).await?;
    // 2. Calculate the total price of items in the cart.
    let total_price = calculate_total_price(&cart_items);

    // 3. Apply discounts if a valid coupon code is provided.
    let discount_amount = apply_discount(&coupon_code);

    // 4. Calculate the final price after applying discounts.
    let final_price = total_price - discount_amount;

    // 5. Create an order in the database.
    let order_id = create_order_in_db(pool.get_ref(), user_id, final_price.to_f64().unwrap()).await?;

    // 6. Clear the user's cart in the database (remove items).
    clear_cart_items_in_db(pool.get_ref(), user_id).await?;

    // 7. Return a response indicating a successful checkout.
    Ok(HttpResponse::Ok().body(format!("Checkout completed. Order ID: {}", order_id)))
}

async fn fetch_cart_items_from_db(pool: &PgPool, user_id: i32) -> Result<Vec<CartItem>, HttpResponse> {
    // Define the SQL query to retrieve cart items for the given user_id.
    let query = sqlx::query_as!(
        CartItem,
        r#"
        SELECT id, user_id, product_id, quantity, price
        FROM cart_items
        WHERE user_id = $1
        "#,
        user_id
    );

    // Execute the query and fetch cart items from the database.
    match query.fetch_all(pool).await {
        Ok(cart_items) => Ok(cart_items), // Return the fetched cart items.
        Err(_) => {
            // Handle the error if fetching fails.
            Err(HttpResponse::InternalServerError().body("Failed to fetch cart items"))
        }
    }
}



async fn clear_cart_items_in_db(pool: &PgPool, user_id: i32) -> Result<(), HttpResponse> {
    // Define the SQL query to delete cart items for the given user_id.
    let query = sqlx::query!(
        "DELETE FROM cart_items WHERE user_id = $1",
        user_id
    );

    // Execute the query to delete cart items.
    match query.execute(pool).await {
        Ok(_) => Ok(()), // Successfully cleared cart items.
        Err(_) => {
            // Handle the error if clearing cart items fails.
            Err(HttpResponse::InternalServerError().body("Failed to clear cart items"))
        }
    }
}


fn calculate_total_price(cart_items: &[CartItem]) -> Decimal {
    let mut total_price = Decimal::zero(); // Initialize a Decimal with zero value

    for item in cart_items {
        // Calculate the total price for each item and accumulate it
        let item_price = item.price;
        let item_total = item_price * Decimal::from(item.quantity);
        total_price += item_total;
    }

    total_price
}

fn apply_discount(coupon_code: &str) -> Decimal {
    // let's assume we have a hard-coded list of valid coupons.

    let valid_coupons = [
        ("DISCOUNT_CODE_1", Decimal::from_f64(10.0).unwrap()), // $10 discount
        ("DISCOUNT_CODE_2", Decimal::from_f64(5.0).unwrap()),  // $5 discount
    ];

    // Check if the provided coupon code exists in the list of valid coupons.
    if let Some(&(_, discount_amount)) = valid_coupons.iter().find(|&&(code, _)| code == coupon_code) {
        return discount_amount;
    }

    // If the coupon code is not found or is invalid, return zero as no discount.
    Decimal::zero()
}

async fn create_order_in_db(pool: &PgPool, user_id: i32, final_price: f64) -> Result<i32, HttpResponse> {
    // Define the SQL query to insert a new order.
    let query = sqlx::query!(
        r#"
        INSERT INTO orders (user_id, total_price, status)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        user_id,
        final_price,
        "Pending"  // Set the initial status as "Pending" or adjust as needed.
    );

    // Execute the query and fetch the inserted order_id.
    match query.fetch_one(pool).await {
        Ok(order) => {
            // Extract the order_id from the query result and return it.
            let order_id = order.id;
            Ok(order_id)
        }
        Err(_) => {
            // Handle the error if order creation fails.
            Err(HttpResponse::InternalServerError().body("Failed to create an order"))
        }
    }
}

async fn get_coupons(pool: web::Data<PgPool>) -> impl Responder {
    // Define the SQL query to retrieve available discount coupons.
    let query = sqlx::query_as!(
        DiscountCoupon,
        r#"
        SELECT id, code, discount_amount, expiration_date
        FROM discount_coupons
        WHERE expiration_date >= CURRENT_DATE
        ORDER BY expiration_date
        "#
    );

    // Execute the query and fetch the list of available coupons from the database.
    match query.fetch_all(pool.get_ref()).await {
        Ok(coupons) => {
            // Return a response with the list of available coupons as a JSON array.
            HttpResponse::Ok().json(coupons)
        }
        Err(_) => {
            // Handle the error if fetching coupons fails.
            HttpResponse::InternalServerError().body("Failed to fetch discount coupons")
        }
    }
}


