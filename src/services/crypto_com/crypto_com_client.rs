use crate::services::provider::ProviderClient;
use crate::services::types::{OrderBookResponse, ProviderBookRequest, ProviderError};
use envy;
use reqwest;
use serde::Deserialize;

// Crypto.com-specific client for fetching order book data.
// Implements the ProviderClient trait to integrate with the multi-provider system.
pub struct CryptoComClient {
    client: reqwest::Client,
    config: Config,
}

// Configuration for Crypto.com API credentials and endpoints.
// Loaded from environment variables.
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "crypto_com_api_key")]
    pub api_key: String,
    #[serde(rename = "crypto_com_api_secret")]
    pub api_secret: String,
    #[serde(rename = "crypto_com_env", default = "Config::default_env")]
    pub environment: String,
    #[serde(rename = "crypto_com_api_url")]
    pub api_url: String,
}

impl Config {
    // Default environment is testnet for safety.
    fn default_env() -> String {
        "testnet".into()
    }
}

// Response structure for Crypto.com get-book API endpoint.
#[derive(Debug, Deserialize)]
struct CryptoComGetOrderBookResponse {
    code: i32,
    method: Option<String>,
    result: Option<CryptoComGetBookResult>,
    message: Option<String>,
}

// Result data from the Crypto.com order book response.
#[derive(Debug, Deserialize)]
struct CryptoComGetBookResult {
    data: Vec<CryptoComGetBookData>,
    instrument_name: String,
}

// Individual order book data entry containing bids and asks.
#[derive(Debug, Deserialize)]
struct CryptoComGetBookData {
    asks: Vec<Vec<String>>,
    bids: Vec<Vec<String>>,
}

impl CryptoComClient {
    // Creates a new Crypto.com client with configuration from environment variables.
    // Panics if configuration cannot be loaded or client cannot be created.
    pub fn new() -> Self {
        log::info!("Initializing Crypto.com client");

        let config: Config = match envy::from_env::<Config>() {
            Ok(cfg) => {
                log::debug!(
                    "Successfully loaded Crypto.com configuration for environment: {}",
                    cfg.environment
                );
                cfg
            }
            Err(error) => {
                log::error!("Failed to load Crypto.com configuration: {}", error);
                std::process::exit(1);
            }
        };

        log::debug!("Creating HTTP client for Crypto.com API");
        let client = reqwest::Client::builder().build().unwrap_or_else(|e| {
            log::error!("Failed to create Crypto.com HTTP client: {}", e);
            panic!("Failed to create Crypto.com client: {}", e)
        });

        log::info!("Crypto.com client initialized successfully");
        Self { client, config }
    }

    // Fetches the latest order book from Crypto.com API.
    // Handles HTTP requests, response parsing, and data transformation.
    pub async fn fetch_latest_order_book(
        &self,
        request: ProviderBookRequest,
    ) -> Result<OrderBookResponse, ProviderError> {
        let pair = format!("{}_{}", request.base, request.quote);
        log::info!(
            "Fetching order book from Crypto.com for pair: {}, depth: {:?}",
            pair,
            request.depth
        );

        let path = "public/get-book";
        let url = format!("{}/{}", self.config.api_url.trim_end_matches('/'), path);
        log::debug!("Constructed API URL: {}", url);

        let mut query_params = vec![("instrument_name", pair.clone())];
        if let Some(d) = request.depth {
            query_params.push(("depth", d.to_string()));
        }
        log::debug!("Query parameters: {:?}", query_params);

        // Make HTTP GET request
        log::debug!("Sending GET request to Crypto.com API");
        let http_response = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .map_err(|e| {
                log::error!("HTTP request failed for Crypto.com API: {}", e);
                ProviderError::BackendError(e.to_string())
            })?;

        // Parse JSON response
        log::debug!("Parsing JSON response from Crypto.com");
        let response: CryptoComGetOrderBookResponse = http_response.json().await.map_err(|e| {
            log::error!("Failed to parse Crypto.com response JSON: {}", e);
            ProviderError::BackendError(format!("failed to parse response: {}", e))
        })?;

        // Check for API errors
        if response.code != 0 {
            let error_message = response.message.as_deref().unwrap_or("unknown error");
            let method = response.method.as_deref().unwrap_or("unknown method");
            log::error!(
                "Crypto.com API returned error code={} method={} message={}",
                response.code,
                method,
                error_message
            );
            return Err(ProviderError::BackendError(format!(
                "crypto.com error code={} method={} message={}",
                response.code, method, error_message
            )));
        }

        // Extract result data
        let result = response.result.ok_or_else(|| {
            log::error!("Crypto.com response missing result field");
            ProviderError::BackendError("crypto.com response missing result".to_string())
        })?;

        // Get the first book data entry
        let book_data = result.data.into_iter().next().ok_or_else(|| {
            log::error!("Crypto.com response missing order book data");
            ProviderError::BackendError("missing order book data".to_string())
        })?;

        // Process bids
        log::debug!(
            "Processing {} bids from Crypto.com response",
            book_data.bids.len()
        );
        let bids = book_data
            .bids
            .into_iter()
            .filter_map(|item| {
                if item.len() >= 2 {
                    Some((item[0].clone(), item[1].clone()))
                } else {
                    log::warn!("Skipping malformed bid entry: {:?}", item);
                    None
                }
            })
            .collect();

        // Process asks
        log::debug!(
            "Processing {} asks from Crypto.com response",
            book_data.asks.len()
        );
        let asks = book_data
            .asks
            .into_iter()
            .filter_map(|item| {
                if item.len() >= 2 {
                    Some((item[0].clone(), item[1].clone()))
                } else {
                    log::warn!("Skipping malformed ask entry: {:?}", item);
                    None
                }
            })
            .collect();

        log::info!(
            "Successfully processed order book from Crypto.com for pair: {}",
            pair
        );
        Ok(OrderBookResponse {
            provider: "crypto_com".to_string(),
            pair: result.instrument_name,
            bids,
            asks,
        })
    }
}

impl ProviderClient for CryptoComClient {
    // Implementation of ProviderClient trait for Crypto.com.
    // Wraps the internal fetch method in a pinned future.
    fn get_latest_order_book(
        &self,
        request: ProviderBookRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<OrderBookResponse, ProviderError>> + Send + '_>,
    > {
        log::debug!(
            "ProviderClient trait called for Crypto.com with request: {:?}",
            request
        );
        Box::pin(async move { self.fetch_latest_order_book(request).await })
    }
}
