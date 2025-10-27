use actix_web::{web, App, HttpServer};
use sqlx::mysql::MySqlPoolOptions;
use std::io::Result as IoResult;

// Declare all the modules
mod config;
mod db;
mod error;
mod external;
mod image;
mod models;
mod routes;

use routes::AppState;

#[tokio::main]
async fn main() -> IoResult<()> {
    // Load configuration from .env
    dotenvy::dotenv().ok();
    let config = config::Config::from_env();

    // Setup logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // (Optional) Run SQLx migrations
    // let db_pool_for_migration = MySqlPoolOptions::new()
    //     .connect(&config.database_url)
    //     .await
    //     .expect("Failed to connect for migrations");
    // sqlx::migrate!("./migrations")
    //     .run(&db_pool_for_migration)
    //     .await
    //     .expect("Failed to run database migrations");
    // db_pool_for_migration.close().await;

    // Setup database connection pool
    let db_pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database connection pool");

    log::info!("Database connection pool established");

    // Run migrations at startup
    log::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run database migrations");
    log::info!("Database migrations complete.");

    // Setup shared application state
    let app_state = web::Data::new(AppState {
        db_pool,
        http_client: reqwest::Client::new(),
    });

    let server_address = format!("0.0.0.0:{}", config.port);
    log::info!("Starting server at http://{}", server_address);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .configure(routes::configure_routes) // Configure routes from routes.rs
    })
    .bind(server_address)?
    .run()
    .await
}