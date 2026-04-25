use crate::middleware::middleware;
use crate::services::provider::Clients;
use axum::Router;
use envy;
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;

// Application state shared across all routes and handlers.
// Contains the client registry for different crypto providers.
#[derive(Clone)]
pub struct AppState {
    pub clients: Arc<Clients>,
}

// Configuration structure for server settings.
// Loaded from environment variables with default fallbacks.
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "app_host", default = "Config::default_host")]
    pub host: String,
    #[serde(rename = "app_port", default = "Config::default_port")]
    pub port: u16,
}

impl Config {
    // Default host for the server if not specified in environment.
    pub fn default_host() -> String {
        "127.0.0.1".to_string()
    }

    // Default port for the server if not specified in environment.
    pub fn default_port() -> u16 {
        8080
    }
}

// Main application runner function.
// Initializes configuration, clients, and starts the HTTP server.
pub async fn run() {
    log::info!("Loading server configuration from environment variables");

    // Load environment variables into config struct
    let config: Config = match envy::from_env::<Config>() {
        Ok(cfg) => {
            log::info!(
                "Successfully loaded configuration: host={}, port={}",
                cfg.host,
                cfg.port
            );
            cfg
        }
        Err(error) => {
            log::error!("Failed to load server configuration: {}", error);
            std::process::exit(1);
        }
    };

    log::debug!("Initializing tracing subscriber for logging");

    log::info!("Initializing crypto provider clients");
    // Initialize clients once and share via Arc for thread safety
    let clients = Arc::new(Clients::new());
    let state = AppState { clients };

    log::debug!("Building Axum router with routes and middleware");
    let app = Router::new()
        .merge(crate::routes::router())
        .layer(axum::middleware::from_fn(middleware::request_log))
        .with_state(state);

    log::info!("Binding TCP listener to {}:{}", config.host, config.port);
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap_or_else(|err| {
            log::error!("Failed to bind TCP listener: {}", err);
            std::process::exit(1);
        });

    let local_addr = listener.local_addr().unwrap();
    log::info!("Server successfully started and running on {}", local_addr);

    // Start serving requests - this will run indefinitely
    if let Err(err) = axum::serve(listener, app).await {
        log::error!("Server encountered an error: {}", err);
        std::process::exit(1);
    }
}
