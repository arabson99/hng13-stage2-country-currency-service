use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

// --- External API Response Models ---

#[derive(Debug, Deserialize)]
pub struct RestCountryCurrency {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct RestCountryResponse {
    pub name: String,
    pub capital: Option<String>,
    pub region: Option<String>,
    pub population: i64,
    pub flag: Option<String>,
    pub currencies: Option<Vec<RestCountryCurrency>>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRateResponse {
    pub rates: HashMap<String, f64>,
}

// --- Database & Internal Models ---

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct Country {
    pub id: i32,
    pub name: String,
    pub capital: Option<String>,
    pub region: Option<String>,
    pub population: i64,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<sqlx::types::Decimal>,
    pub estimated_gdp: Option<sqlx::types::Decimal>,
    pub flag_url: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AppStatus {
    pub total_countries: i32,
    pub last_refreshed_at: Option<DateTime<Utc>>,
}

// --- API Query Parameters ---

#[derive(Debug, Deserialize)]
pub struct GetCountriesQuery {
    pub region: Option<String>,
    pub currency: Option<String>,
    pub sort: Option<String>,
}

// --- API Response Models ---

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub status: String,
    pub countries_processed: usize,
    pub last_refreshed_at: DateTime<Utc>,
}