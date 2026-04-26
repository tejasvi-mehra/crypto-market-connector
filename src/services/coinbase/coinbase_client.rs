use crate::services::provider::ProviderClient;
use crate::services::types::{OrderBookResponse, ProviderBookRequest, ProviderError};
use coinbase_advanced::models::GetProductBookParams;
use envy;
use serde::Deserialize;

// Coinbase-specific client for fetching order book data.
// Implements the ProviderClient trait to integrate with the multi-provider system.
// Uses the Coinbase Advanced API for real-time market data.
pub struct CoinbaseClient {
    // The REST client for communicating with Coinbase API
    client: coinbase_advanced::RestClient,
}

// Configuration for Coinbase API client setup.
// Supports both production and sandbox (testnet) environments.
// Authentication credentials are loaded from environment variables by the Coinbase SDK.
#[derive(Debug, Deserialize)]
pub struct Config {
    // Environment setting: "testnet" for sandbox, "production" for live trading
    // Default: "testnet" for safety
    #[serde(rename = "coinbase_env", default = "Config::default_env")]
    pub environment: String,
}

impl Config {
    // Default environment is testnet for safety.
    // Prevents accidental connections to production when environment not specified.
    fn default_env() -> String {
        "testnet".into()
    }
}

impl CoinbaseClient {
    // Creates a new Coinbase client instance.
    //
    // This method:
    // - Loads configuration from environment variables
    // - Creates Coinbase SDK credentials from environment (CB_ACCESS_KEY, CB_SECRET_KEY, CB_PASSPHRASE)
    // - Initializes the REST client with appropriate sandbox/production settings
    // - Logs all initialization steps
    //
    // # Panics
    // Panics if configuration cannot be loaded, credentials are missing, or client creation fails.
    pub fn new() -> Self {
        log::info!("Initializing Coinbase client");

        // Load environment configuration for Coinbase client
        let config: Config = match envy::from_env::<Config>() {
            Ok(cfg) => {
                log::debug!(
                    "Successfully loaded Coinbase configuration for environment: {}",
                    cfg.environment
                );
                cfg
            }
            Err(error) => {
                log::error!("Failed to load Coinbase configuration: {}", error);
                std::process::exit(1);
            }
        };

        // Determine if sandbox or production based on configuration
        let sandbox = match config.environment.as_str() {
            "testnet" => {
                log::debug!("Configuring Coinbase client for testnet/sandbox environment");
                true
            }
            "production" => {
                log::warn!("Configuring Coinbase client for PRODUCTION environment");
                false
            }
            _ => {
                log::error!(
                    "Invalid Coinbase environment: {}. Use 'testnet' or 'production'.",
                    config.environment
                );
                std::process::exit(1);
            }
        };

        // Load API credentials from environment variables
        // Requires: CB_ACCESS_KEY, CB_SECRET_KEY, CB_PASSPHRASE
        log::debug!("Loading Coinbase API credentials from environment");
        let coinbase_credentials = match coinbase_advanced::Credentials::from_env() {
            Ok(cfg) => {
                log::debug!("Successfully loaded Coinbase credentials from environment");
                cfg
            }
            Err(error) => {
                log::error!("Failed to load Coinbase credentials: {}", error);
                log::error!("Ensure CB_ACCESS_KEY, CB_SECRET_KEY, and CB_PASSPHRASE are set");
                std::process::exit(1);
            }
        };

        // Create REST client with configured credentials and environment
        log::debug!(
            "Creating REST API client for Coinbase (sandbox={})",
            sandbox
        );
        let client = coinbase_advanced::RestClient::builder()
            .credentials(coinbase_credentials)
            .sandbox(sandbox)
            .build()
            .unwrap_or_else(|e| {
                log::error!("Failed to create Coinbase REST client: {}", e);
                panic!("Failed to create Coinbase REST client: {}", e)
            });

        log::info!("Coinbase client initialized successfully");

        CoinbaseClient { client }
    }

    // Fetches the latest order book data from Coinbase for a trading pair.
    //
    // This method:
    // - Formats the trading pair in Coinbase format (BASE-QUOTE)
    // - Queries the product book endpoint with specified depth
    // - Transforms the response into the standard OrderBookResponse format
    //
    // # Arguments
    // - `request`: ProviderBookRequest containing base, quote, and optional depth
    //
    // # Returns
    // - Ok(OrderBookResponse): Contains formatted bids and asks data
    // - Err(ProviderError::BackendError): If API calls fail
    async fn fetch_latest_order_book(
        &self,
        request: ProviderBookRequest,
    ) -> Result<OrderBookResponse, ProviderError> {
        // Format pair in Coinbase style: BTC-USDT
        let pair = format!("{}-{}", request.base, request.quote);
        log::info!(
            "Fetching order book from Coinbase for pair: {}, depth: {:?}",
            pair,
            request.depth
        );

        // Build parameters for the API request
        let mut params = GetProductBookParams::new(pair.clone());

        // Apply depth limit if specified
        if let Some(depth) = request.depth {
            log::debug!("Setting book depth limit to {}", depth);
            params = params.limit(depth);
        }

        // Fetch the product order book
        log::debug!("Requesting product book from Coinbase API: {}", pair);
        let response = self
            .client
            .public()
            .get_product_book(params)
            .await
            .map_err(|e| {
                log::error!("Coinbase API product book call failed for {}: {}", pair, e);
                ProviderError::BackendError(format!("Failed to fetch product book: {}", e))
            })?;

        // Extract and transform data from response
        log::debug!(
            "Extracting order book data from Coinbase response: {} bids, {} asks",
            response.bids.len(),
            response.asks.len()
        );

        // Transform bids from Coinbase format to standard format (price, size)
        let bids: Vec<(String, String)> = response
            .bids
            .iter()
            .map(|bid| {
                log::trace!("Processing bid: price={}, size={}", bid.price, bid.size);
                (bid.price.clone(), bid.size.clone())
            })
            .collect();

        // Transform asks from Coinbase format to standard format (price, size)
        let asks: Vec<(String, String)> = response
            .asks
            .iter()
            .map(|ask| {
                log::trace!("Processing ask: price={}, size={}", ask.price, ask.size);
                (ask.price.clone(), ask.size.clone())
            })
            .collect();

        log::info!(
            "Successfully processed Coinbase order book for pair: {}",
            pair
        );

        // Return formatted response
        Ok(OrderBookResponse {
            provider: "coinbase".to_string(),
            pair: pair.clone(),
            bids,
            asks,
        })
    }
}

impl ProviderClient for CoinbaseClient {
    // Implementation of the ProviderClient trait for Coinbase.
    // Wraps the internal fetch method in a pinned async future for compatibility
    // with the provider registry system.
    fn get_latest_order_book(
        &self,
        request: ProviderBookRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<OrderBookResponse, ProviderError>> + Send + '_>,
    > {
        log::debug!(
            "ProviderClient trait called for Coinbase with request: {:?}",
            request
        );
        Box::pin(async move { self.fetch_latest_order_book(request).await })
    }
}
