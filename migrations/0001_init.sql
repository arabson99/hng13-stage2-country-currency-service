-- Create the main table for storing country data
CREATE TABLE IF NOT EXISTS `countries` (
  `id` INT AUTO_INCREMENT PRIMARY KEY,
  `name` VARCHAR(255) NOT NULL UNIQUE,
  `capital` VARCHAR(255) NULL,
  `region` VARCHAR(255) NULL,
  `population` BIGINT NOT NULL,
  `currency_code` VARCHAR(10) NULL,
  `exchange_rate` DECIMAL(20, 6) NULL,
  `estimated_gdp` DECIMAL(30, 6) NULL,
  `flag_url` VARCHAR(2048) NULL,
  `last_refreshed_at` TIMESTAMP NOT NULL
);

-- Create a table to store the global application status
CREATE TABLE IF NOT EXISTS `app_status` (
  `id` INT PRIMARY KEY DEFAULT 1, -- Only one row will exist
  `total_countries` INT NOT NULL DEFAULT 0,
  `last_refreshed_at` TIMESTAMP NULL
);

-- Insert the initial status row
INSERT IGNORE INTO `app_status` (id) VALUES (1);