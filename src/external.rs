use crate::error::AppError;
use crate::models::{ExchangeRateResponse, RestCountryResponse};

const COUNTRIES_API_URL: &str = "https://restcountries.com/v2/all?fields=name,capital,region,population,flag,currencies";
const EXCHANGE_RATE_API_URL: &str = "https://open.er-api.com/v6/latest/USD";

/// Fetches all country data from the RestCountries API.
pub async fn fetch_countries(
    client: &reqwest::Client,
) -> Result<Vec<RestCountryResponse>, AppError> {
    client
        .get(COUNTRIES_API_URL)
        .send()
        .await
        .map_err(|e| AppError::ApiError {
            source: e,
            api_name: "RestCountries".to_string(),
        })?
        .json::<Vec<RestCountryResponse>>()
        .await
        .map_err(|e| AppError::ApiError {
            source: e,
            api_name: "RestCountries (parsing)".to_string(),
        })
}

/// Fetches the latest USD exchange rates.
pub async fn fetch_exchange_rates(
    client: &reqwest::Client,
) -> Result<ExchangeRateResponse, AppError> {
    client
        .get(EXCHANGE_RATE_API_URL)
        .send()
        .await
        .map_err(|e| AppError::ApiError {
            source: e,
            api_name: "OpenExchangeRates".to_string(),
        })?
        .json::<ExchangeRateResponse>()
        .await
        .map_err(|e| AppError::ApiError {
            source: e,
            api_name: "OpenExchangeRates (parsing)".to_string(),
        })
}