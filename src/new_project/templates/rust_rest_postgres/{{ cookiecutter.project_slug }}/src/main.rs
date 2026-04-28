mod config;
mod error;
mod models;
mod routes;
mod state;

use std::error::Error;

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

use crate::{config::Config, routes::router, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let app_state = AppState {
        config: config.clone(),
        pool,
    };
    let app = router(app_state);
    let address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&address).await?;

    println!("listening on {address}");

    axum::serve(listener, app).await?;

    Ok(())
}
