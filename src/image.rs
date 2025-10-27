use crate::error::AppError;
use crate::models::{AppStatus, Country};
use ab_glyph::{FontRef, PxScale};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use std::fs;
use std::path::Path;

pub const SUMMARY_IMAGE_PATH: &str = "cache/summary.png";
const FONT_PATH: &str = "./DejaVuSans.ttf"; // Assumes font is in project root

/// Generates and saves a summary image.
pub fn generate_summary_image(
    status: &AppStatus,
    top_countries: &[Country],
) -> Result<(), AppError> {
    // Ensure cache directory exists
    if let Some(parent) = Path::new(SUMMARY_IMAGE_PATH).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::ImageError(format!("Failed to create cache dir: {}", e)))?;
    }

    // Load font
    let font_data = fs::read(FONT_PATH)
        .map_err(|e| AppError::ImageError(format!("Failed to read font file '{}': {}", FONT_PATH, e)))?;
    let font = FontRef::try_from_slice(&font_data)
        .map_err(|e| AppError::ImageError(format!("Failed to parse font: {}", e)))?;

    // Create image
    let (width, height) = (600, 400);
    let mut img = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));
    let text_color = Rgb([0, 0, 0]);
    let scale_large = PxScale::from(32.0);
    let scale_medium = PxScale::from(24.0);
    let scale_small = PxScale::from(18.0);

    let mut y_pos = 20;

    // Title
    draw_text_mut(
        &mut img,
        text_color,
        20,
        y_pos,
        scale_large,
        &font,
        "Country Data Summary",
    );
    y_pos += 40;

    // Status
    draw_text_mut(
        &mut img,
        text_color,
        20,
        y_pos,
        scale_medium,
        &font,
        &format!("Total Countries: {}", status.total_countries),
    );
    y_pos += 30;

    let timestamp_str = status
        .last_refreshed_at
        .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Never".to_string());
    draw_text_mut(
        &mut img,
        text_color,
        20,
        y_pos,
        scale_medium,
        &font,
        &format!("Last Refresh: {}", timestamp_str),
    );
    y_pos += 50;

    // Top 5
    draw_text_mut(
        &mut img,
        text_color,
        20,
        y_pos,
        scale_medium,
        &font,
        "Top 5 by Estimated GDP:",
    );
    y_pos += 30;

    for (i, country) in top_countries.iter().enumerate() {
        let gdp_str = country
            .estimated_gdp
            .map(|gdp| format!("${}", gdp.round_dp(2)))
            .unwrap_or_else(|| "N/A".to_string());

        let line = format!("{}. {} ({})", i + 1, country.name, gdp_str);
        draw_text_mut(&mut img, text_color, 30, y_pos, scale_small, &font, &line);
        y_pos += 25;
    }

    // Save image
    img.save(SUMMARY_IMAGE_PATH)
        .map_err(|e| AppError::ImageError(format!("Failed to save image: {}", e)))?;

    log::info!("Summary image generated at {}", SUMMARY_IMAGE_PATH);
    Ok(())
}