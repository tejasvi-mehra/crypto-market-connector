use crypto_market_connector::app::run;

#[tokio::main]
async fn main() {
    // Initialize the logger for canonical logging throughout the application
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .try_init()
        .unwrap_or_else(|_| {
            log::warn!("Tracing subscriber already initialized, skipping");
        });

    tracing::info!("Starting Crypto Market Connector application");
    run().await;
}
