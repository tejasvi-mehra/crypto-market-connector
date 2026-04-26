use crate::services::provider::ProviderClient;
use crate::services::types::{OrderBookResponse, ProviderBookRequest, ProviderError};
use binance_sdk::config::ConfigurationRestApi;
use binance_sdk::spot::{SpotRestApi, rest_api::DepthParams};
use envy;
use serde::Deserialize;

// Binance-specific client for fetching order book data.
// Implements the ProviderClient trait to integrate with the multi-provider system.
pub struct BinanceClient {
    client: binance_sdk::spot::rest_api::RestApi,
}

// Configuration for Binance API credentials and environment.
// Loaded from environment variables.
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "binance_api_key")]
    pub api_key: String,
    #[serde(rename = "binance_api_secret")]
    pub api_secret: String,
    #[serde(rename = "binance_env", default = "Config::default_env")]
    pub environment: String,
}

impl Config {
    // Default environment is testnet for safety.
    fn default_env() -> String {
        "testnet".into()
    }
}

impl BinanceClient {
    // Creates a new Binance client with configuration from environment variables.
    // Panics if configuration cannot be loaded or client cannot be created.
    pub fn new() -> Self {
        log::info!("Initializing Binance client");

        // Load Binance API configuration from environment variables
        let config: Config = match envy::from_env::<Config>() {
            Ok(cfg) => {
                log::debug!(
                    "Successfully loaded Binance configuration for environment: {}",
                    cfg.environment
                );
                cfg
            }
            Err(error) => {
                log::error!("Failed to load Binance configuration: {}", error);
                std::process::exit(1);
            }
        };

        log::debug!("Building Binance REST API configuration");
        let binance_config = ConfigurationRestApi::builder()
            .api_key(config.api_key)
            .api_secret(config.api_secret)
            .build()
            .unwrap_or_else(|e| {
                log::error!("Failed to create Binance configuration: {}", e);
                panic!("Failed to create Binance configuration: {}", e)
            });

        log::debug!(
            "Creating Binance REST API client for environment: {}",
            config.environment
        );
        let client = match config.environment.as_str() {
            "testnet" => {
                log::info!("Using Binance testnet environment");
                SpotRestApi::testnet(binance_config)
            }
            "production" => {
                log::info!("Using Binance production environment");
                SpotRestApi::production(binance_config)
            }
            _ => {
                log::error!(
                    "Invalid Binance environment: {}. Use 'testnet' or 'production'.",
                    config.environment
                );
                std::process::exit(1);
            }
        };

        log::info!("Binance client initialized successfully");
        Self { client }
    }

    // Internal method to fetch the latest order book from Binance.
    // Handles API calls and data transformation.
    async fn fetch_latest_order_book(
        &self,
        request: ProviderBookRequest,
    ) -> Result<OrderBookResponse, ProviderError> {
        let pair = format!("{}{}", request.base, request.quote);
        let depth: Option<i32> = request.depth.map(|d| d as i32);

        log::info!(
            "Fetching order book from Binance for pair: {}, depth: {:?}",
            pair,
            depth
        );

        // Make API call to get depth data
        log::debug!("Making depth API call to Binance for symbol: {}", pair);
        let response = self
            .client
            .depth(DepthParams {
                symbol: pair.clone(),
                limit: depth,
                symbol_status: None,
            })
            .await
            .map_err(|e| {
                log::error!("Binance API depth call failed for {}: {}", pair, e);
                ProviderError::BackendError(e.to_string())
            })?;

        // Extract data from response
        log::debug!("Extracting depth data from Binance response");
        let depth_data = response.data().await.map_err(|e| {
            log::error!("Failed to extract depth data from Binance response: {}", e);
            ProviderError::BackendError(e.to_string())
        })?;

        // Process bids
        log::debug!(
            "Processing {} bids from Binance response",
            depth_data.bids.as_ref().map_or(0, |b| b.len())
        );
        let bids = depth_data
            .bids
            .unwrap_or_default()
            .into_iter()
            .filter_map(|bid| {
                if bid.len() >= 2 {
                    Some((bid[0].clone(), bid[1].clone()))
                } else {
                    log::warn!("Skipping malformed bid entry: {:?}", bid);
                    None
                }
            })
            .collect();

        // Process asks
        log::debug!(
            "Processing {} asks from Binance response",
            depth_data.asks.as_ref().map_or(0, |a| a.len())
        );
        let asks = depth_data
            .asks
            .unwrap_or_default()
            .into_iter()
            .filter_map(|ask| {
                if ask.len() >= 2 {
                    Some((ask[0].clone(), ask[1].clone()))
                } else {
                    log::warn!("Skipping malformed ask entry: {:?}", ask);
                    None
                }
            })
            .collect();

        log::info!(
            "Successfully processed order book from Binance for pair: {}",
            pair
        );
        Ok(OrderBookResponse {
            provider: "binance".to_string(),
            pair: pair.clone(),
            bids,
            asks,
        })
    }
}

impl ProviderClient for BinanceClient {
    // Implementation of ProviderClient trait for Binance.
    // Wraps the internal fetch method in a pinned future.
    fn get_latest_order_book(
        &self,
        request: ProviderBookRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<OrderBookResponse, ProviderError>> + Send + '_>,
    > {
        log::debug!(
            "ProviderClient trait called for Binance with request: {:?}",
            request
        );
        Box::pin(async move { self.fetch_latest_order_book(request).await })
    }
}
