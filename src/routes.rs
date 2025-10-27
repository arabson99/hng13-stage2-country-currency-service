use crate::db;
use crate::error::AppError;
use crate::external;
use crate::image;
use crate::models::{GetCountriesQuery, RefreshResponse};

use actix_files::NamedFile;
use actix_web::{web, HttpResponse, Responder};
use sqlx::MySqlPool;
use std::path::Path;

/// Shared application state
pub struct AppState {
    pub db_pool: MySqlPool,
    pub http_client: reqwest::Client,
}

/// Configures all API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/countries")
            .route("/refresh", web::post().to(refresh_countries))
            .route("", web::get().to(get_countries))
            .route("/image", web::get().to(serve_summary_image))
            .route("/{name}", web::get().to(get_country))
            .route("/{name}", web::delete().to(delete_country)),
    )
    .route("/status", web::get().to(get_status));
}

/// POST /countries/refresh
/// Fetches new data, refreshes the DB, and generates the summary image.
async fn refresh_countries(
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    log::info!("Starting data refresh...");

    // 1. Fetch data from external APIs
    let (countries_res, mut rates_res) = tokio::try_join!(
        external::fetch_countries(&state.http_client),
        external::fetch_exchange_rates(&state.http_client)
    )?;

    log::info!(
        "Fetched {} countries and {} exchange rates",
        countries_res.len(),
        rates_res.rates.len()
    );

    /*// For testing `If currency_code is not found in the exchange rates API:`
    rates_res.rates.remove("NGN");
    log::warn!("TESTING: Removed 'NGN' from rates map.");
    */

    // 2. Process and save data to DB
    let (status, top_countries) =
        db::refresh_data(&state.db_pool, countries_res, rates_res.rates).await?;

    log::info!(
        "Database refresh complete. {} countries processed.",
        status.total_countries
    );

    // 3. Generate summary image
    image::generate_summary_image(&status, &top_countries)?;

    let response = RefreshResponse {
        status: "success".to_string(),
        countries_processed: status.total_countries as usize,
        last_refreshed_at: status.last_refreshed_at.unwrap_or_else(chrono::Utc::now),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// GET /countries
/// Retrieves a list of all countries, with optional filters.
async fn get_countries(
    state: web::Data<AppState>,
    query: web::Query<GetCountriesQuery>,
) -> Result<impl Responder, AppError> {
    let countries = db::get_all_countries(&state.db_pool, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(countries))
}

/// GET /countries/:name
/// Retrieves a single country by its name.
async fn get_country(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let name = path.into_inner();
    let country = db::get_country_by_name(&state.db_pool, &name).await?;
    Ok(HttpResponse::Ok().json(country))
}

/// DELETE /countries/:name
/// Deletes a single country by its name.
async fn delete_country(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let name = path.into_inner();
    db::delete_country_by_name(&state.db_pool, &name).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// GET /status
/// Returns the total country count and last refresh time.
async fn get_status(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let status = db::get_app_status(&state.db_pool).await?;
    Ok(HttpResponse::Ok().json(status))
}

/// GET /countries/image
/// Serves the generated summary.png image.
async fn serve_summary_image() -> Result<impl Responder, AppError> {
    let path_str = image::SUMMARY_IMAGE_PATH;
    if !Path::new(path_str).exists() {
        return Err(AppError::NotFound("Summary image not found. Please run /countries/refresh first.".to_string()));
    }

    NamedFile::open(path_str)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serve image: {}", e)))
}