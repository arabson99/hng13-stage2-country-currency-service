use crate::error::AppError;
use crate::models::{
    AppStatus, Country, GetCountriesQuery, RestCountryResponse,
};
use chrono::{Utc, SubsecRound};
use rand::Rng;
use sqlx::{MySqlPool, QueryBuilder};
use std::collections::HashMap;

/// Processes and caches all country and rate data in the database.
pub async fn refresh_data(
    pool: &MySqlPool,
    countries: Vec<RestCountryResponse>,
    rates: HashMap<String, f64>,
) -> Result<(AppStatus, Vec<Country>), AppError> {
    let mut tx = pool.begin().await?;
    let refresh_time = Utc::now().round_subsecs(1);
    let mut country_count = 0;
    let mut rng = rand::thread_rng();

    for country in countries {
        country_count += 1;

        // 1. Determine Currency Code
        let currency_code = country
            .currencies
            .as_ref()
            .and_then(|c| c.first())
            .map(|c| c.code.clone());

        // 2. Determine Exchange Rate & GDP
        let (exchange_rate, estimated_gdp) = match &currency_code {
            Some(code) => {
                if let Some(rate) = rates.get(code) {
                    // Currency found in rates
                    let random_multiplier: f64 = rng.gen_range(1000.0..=2000.0);
                    let gdp = (country.population as f64 * random_multiplier) / rate;
                    (Some(*rate), Some(gdp))
                } else {
                    // Currency code exists but not in rates API
                    (None, None)
                }
            }
            None => {
                // No currency array or it's empty
                (None, Some(0.0))
            }
        };

        // 3. Upsert logic
        // Use `INSERT ... ON DUPLICATE KEY UPDATE` for efficient upsert
        sqlx::query!(
            r#"
            INSERT INTO countries (
                name, capital, region, population, currency_code, 
                exchange_rate, estimated_gdp, flag_url, last_refreshed_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                capital = VALUES(capital),
                region = VALUES(region),
                population = VALUES(population),
                currency_code = VALUES(currency_code),
                exchange_rate = VALUES(exchange_rate),
                estimated_gdp = VALUES(estimated_gdp),
                flag_url = VALUES(flag_url),
                last_refreshed_at = VALUES(last_refreshed_at)
            "#,
            country.name,
            country.capital,
            country.region,
            country.population,
            currency_code,
            exchange_rate,
            estimated_gdp,
            country.flag,
            refresh_time
        )
        .execute(&mut *tx)
        .await?;
    }

    // 4. Update app status
    sqlx::query!(
        r#"
        UPDATE app_status 
        SET total_countries = ?, last_refreshed_at = ? 
        WHERE id = 1
        "#,
        country_count,
        refresh_time
    )
    .execute(&mut *tx)
    .await?;

    // 5. Fetch top 5 countries for image generation
    let top_countries = sqlx::query_as!(
        Country,
        r#"
        SELECT * FROM countries 
        WHERE estimated_gdp IS NOT NULL
        ORDER BY estimated_gdp DESC 
        LIMIT 5
        "#
    )
    .fetch_all(&mut *tx)
    .await?;

    // 6. Commit transaction
    tx.commit().await?;

    let status = AppStatus {
        total_countries: country_count,
        last_refreshed_at: Some(refresh_time),
    };

    Ok((status, top_countries))
}

/// Retrieves all countries from the DB, supporting filters and sorting.
pub async fn get_all_countries(
    pool: &MySqlPool,
    query: GetCountriesQuery,
) -> Result<Vec<Country>, AppError> {
    let mut qb: QueryBuilder<sqlx::MySql> = QueryBuilder::new("SELECT * FROM countries");
    let mut needs_where = true;

    if let Some(region) = query.region {
        qb.push(" WHERE region = ");
        qb.push_bind(region);
        needs_where = false;
    }

    if let Some(currency) = query.currency {
        if needs_where {
            qb.push(" WHERE currency_code = ");
        } else {
            qb.push(" AND currency_code = ");
        }
        qb.push_bind(currency);
    }

    if let Some(sort) = query.sort {
        let sort_sql = match sort.as_str() {
            "gdp_desc" => Some(" ORDER BY estimated_gdp DESC"),
            "gdp_asc" => Some(" ORDER BY estimated_gdp ASC"),
            "pop_desc" => Some(" ORDER BY population DESC"),
            "pop_asc" => Some(" ORDER BY population ASC"),
            "name_asc" => Some(" ORDER BY name ASC"),
            "name_desc" => Some(" ORDER BY name DESC"),
            _ => None, // Ignore invalid sort key
        };

        if let Some(sql) = sort_sql {
            qb.push(sql);
        }
    }
    let countries = qb.build_query_as().fetch_all(pool).await?;
    Ok(countries)
}

/// Retrieves a single country by its name.
pub async fn get_country_by_name(
    pool: &MySqlPool,
    name: &str,
) -> Result<Country, AppError> {
    sqlx::query_as!(Country, "SELECT * FROM countries WHERE name = ?", name)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Country '{}' not found", name)),
            _ => AppError::DatabaseError(e),
        })
}

/// Deletes a single country by its name.
pub async fn delete_country_by_name(
    pool: &MySqlPool,
    name: &str,
) -> Result<(), AppError> {
    let result = sqlx::query!("DELETE FROM countries WHERE name = ?", name)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        Err(AppError::NotFound(format!("Country '{}' not found", name)))
    } else {
        Ok(())
    }
}

/// Gets the global application status.
pub async fn get_app_status(pool: &MySqlPool) -> Result<AppStatus, AppError> {
    let status = sqlx::query_as!(
        AppStatus,
        "SELECT total_countries, last_refreshed_at FROM app_status WHERE id = 1"
    )
    .fetch_one(pool)
    .await?;
    Ok(status)
}