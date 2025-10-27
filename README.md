# hng13-stage2-country-currency-service

This is a RESTful API built with Rust, Actix-web, and SQLx that fetches country and exchange rate data, computes an estimated GDP, and serves it via a set of CRUD and status endpoints.

## Features

-   **`POST /countries/refresh`**: Fetches data from two external APIs, processes it, and caches it in a MySQL database. Also generates a summary image.
-   **`GET /countries`**: Retrieves all cached countries.
    -   Supports filtering: `?region=Africa`, `?currency=NGN`
    -   Supports sorting: `?sort=gdp_desc`, `?sort=pop_asc`, etc.
-   **`GET /countries/:name`**: Gets a single country by its name.
-   **`DELETE /countries/:name`**: Deletes a country from the cache.
-   **`GET /status`**: Shows the total number of countries and the last refresh timestamp.
-   **`GET /countries/image`**: Serves a dynamically generated summary image (total countries, top 5 by GDP, last refresh).

## Tech Stack

-   **Backend**: [Rust](https://www.rust-lang.org/)
-   **Web Framework**: [Actix-web](https://actix.rs/)
-   **Database**: [MySQL](https://www.mysql.com/)
-   **Async SQL**: [SQLx](https://github.com/launchbadge/sqlx)
-   **HTTP Client**: [Reqwest](https://github.com/seanmonstar/reqwest)
-   **Image Generation**: [image-rs](https://github.com/image-rs/image)
-   **Config**: `dotenvy`
-   **Error Handling**: `thiserror`

---

## Setup Instructions

### Prerequisites

1.  **Rust**: Install the Rust toolchain: [https://rustup.rs/](https://rustup.rs/)
2.  **MySQL**: A running MySQL server instance (e.g., local, Docker).
3.  **Font**: Download the `DejaVuSans.ttf` font file. You can get it from [here](https://github.com/dejavu-fonts/dejavu-fonts/blob/master/ttf/DejaVuSans.ttf) (click "Download raw file"). Place this file in the **root** of your project directory.

### 1. Clone Repository

```bash
git clone [https://github.com/your-username/your-repo-name.git](https://github.com/your-username/your-repo-name.git)
cd your-repo-name
```

### 2. Setup Database

1.  Log in to your MySQL server.
2.  Create a new database for the project.

    ```sql
    CREATE DATABASE country_db;
    ```

3.  Use the new database.

    ```sql
    USE country_db;
    ```

4.  Run the `schema.sql` script provided in this repository to create the tables.

    ```bash
    # Example using mysql client:
    mysql -u root -p country_db < schema.sql
    ```

### 3. Configure Environment

1.  Create a `.env` file in the project root.
2.  Copy the contents of `.env.example` into your new `.env` file.
3.  Update the `DATABASE_URL` with your MySQL credentials.

    **`.env` file:**

    ```ini
    # Server Configuration
    PORT=8080
    
    # MySQL Database URL
    # Format: mysql://user:password@host:port/database
    DATABASE_URL=mysql://root:mysecretpassword@127.0.0.1:3306/country_db
    
    # Logging Level
    RUST_LOG=info
    ```

### 4. Build and Run

1.  **Build** the project (in release mode for best performance).

    ```bash
    cargo build --release
    ```

2.  **Run** the server.

    ```bash
    cargo run --release
    ```

The API will now be running on `http://127.0.0.1:8080`.

---

## API Endpoints

### Refresh Data

**POST** `/countries/refresh`

Triggers a full refresh from the external APIs.

**Success Response (200 OK):**

```json
{
  "status": "success",
  "countries_processed": 250,
  "last_refreshed_at": "2025-10-25T14:30:00Z"
}
```

**Failure Response (503 Service Unavailable):**

```json
{
  "error": "External data source unavailable",
  "details": "Could not fetch data from RestCountries: ..."
}
```

### Get All Countries

**GET** `/countries`

**GET** `/countries?region=Africa&sort=gdp_desc`

**Response (200 OK):**

```json
[
  {
    "id": 1,
    "name": "Nigeria",
    "capital": "Abuja",
    "region": "Africa",
    "population": 206139589,
    "currency_code": "NGN",
    "exchange_rate": "1600.230000",
    "estimated_gdp": "25767448125.200000",
    "flag_url": "[https://flagcdn.com/ng.svg](https://flagcdn.com/ng.svg)",
    "last_refreshed_at": "2025-10-25T14:30:00Z"
  }
]
```

### Get Single Country

**GET** `/countries/Nigeria`

**Response (200 OK):** (Returns the single country object)

**Response (404 Not Found):**

```json
{
  "error": "Country 'Nigeria' not found"
}
```

### Delete Country

**DELETE** `/countries/Nigeria`

**Response (204 No Content):** (Empty body)

### Get Status

**GET** `/status`

**Response (200 OK):**

```json
{
  "total_countries": 250,
  "last_refreshed_at": "2025-10-25T14:30:00Z"
}
```

### Get Summary Image

**GET** `/countries/image`

**Response (200 OK):**
Serves the `image/png` file.

**Response (404 Not Found):**

```json
{
  "error": "Summary image not found. Please run /countries/refresh first."
}
```