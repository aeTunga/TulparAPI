use tulpar_api::{config::Config, create_router, db, AppState};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tulpar_api=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    info!(host = %config.host, port = %config.port, "Starting server");

    let pool = db::establish_connection(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    let addr = config.socket_addr();
    let state = Arc::new(AppState::new(pool, config.storage_path));
    let app = create_router(state);

    let listener = TcpListener::bind(&addr).await?;
    info!(addr = %addr, "Listening");

    axum::serve(listener, app).await?;

    Ok(())
}
