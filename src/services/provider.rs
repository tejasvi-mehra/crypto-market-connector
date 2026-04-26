use crate::services::binance::binance_client::BinanceClient;
use crate::services::coinbase::coinbase_client::CoinbaseClient;
use crate::services::crypto_com::crypto_com_client::CryptoComClient;
use crate::services::types::{
    OrderBookRequest, OrderBookResponse, ProviderBookRequest, ProviderError,
};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

// Trait defining the interface for crypto provider clients.
// All provider implementations must implement this trait to be used in the system.
pub trait ProviderClient {
    // Fetches the latest order book data for the given trading pair.
    // Returns a pinned future that resolves to the order book response or an error.
    fn get_latest_order_book(
        &self,
        request: ProviderBookRequest,
    ) -> Pin<Box<dyn Future<Output = Result<OrderBookResponse, ProviderError>> + Send + '_>>;
}

// Registry of all available crypto provider clients.
// Provides thread-safe access to different exchange clients through a hash map.
pub struct Clients {
    client_map: HashMap<String, Box<dyn ProviderClient + Send + Sync>>,
}

impl Clients {
    // Creates a new Clients registry with all supported providers initialized.
    pub fn new() -> Self {
        log::info!("Initializing provider clients registry");

        let mut client_map: HashMap<String, Box<dyn ProviderClient + Send + Sync>> = HashMap::new();

        log::debug!("Adding Binance client to registry");
        client_map.insert("binance".to_string(), Box::new(BinanceClient::new()));

        log::debug!("Adding Crypto.com client to registry");
        client_map.insert("crypto.com".to_string(), Box::new(CryptoComClient::new()));

        log::debug!("Adding Coinbase client to registry");
        client_map.insert("coinbase".to_string(), Box::new(CoinbaseClient::new()));

        log::info!(
            "Successfully initialized {} provider clients",
            client_map.len()
        );
        Self { client_map }
    }

    // Retrieves a provider client by name.
    // Returns None if the provider is not supported.
    pub fn get(&self, provider: &str) -> Option<&Box<dyn ProviderClient + Send + Sync>> {
        log::debug!("Looking up provider client: {}", provider);
        self.client_map.get(provider)
    }

    // Returns an iterator over all registered provider clients.
    // Useful for operations that need to query all providers.
    pub fn providers(
        &self,
    ) -> impl Iterator<Item = (&String, &Box<dyn ProviderClient + Send + Sync>)> {
        self.client_map.iter()
    }
}

// Fetches order book data from a specific provider.
// Dispatches the request to the appropriate provider client based on the provider name.
pub async fn fetch_order_book(
    clients: &Clients,
    request: OrderBookRequest,
) -> Result<OrderBookResponse, ProviderError> {
    let provider_key = request.provider.to_lowercase();
    log::info!(
        "Fetching order book for provider: {}, pair: {}/{}, depth: {:?}",
        provider_key,
        request.base,
        request.quote,
        request.depth
    );

    let client = clients.get(&provider_key).ok_or_else(|| {
        log::error!("Unsupported provider requested: {}", provider_key);
        ProviderError::UnsupportedProvider(provider_key.clone())
    })?;

    log::debug!("Dispatching request to {} provider client", provider_key);
    let result = client
        .get_latest_order_book(ProviderBookRequest {
            base: request.base,
            quote: request.quote,
            depth: request.depth,
        })
        .await;

    match &result {
        Ok(_) => log::info!("Successfully fetched order book from {}", provider_key),
        Err(e) => log::error!("Failed to fetch order book from {}: {}", provider_key, e),
    }

    result
}

// Fetches order book data from all available providers concurrently.
// Returns a map of provider names to their respective results (success or error).
pub async fn fetch_all_order_books(
    clients: &Clients,
    request: ProviderBookRequest,
) -> std::collections::HashMap<String, super::types::ProviderBookResult> {
    log::info!(
        "Fetching order books from all providers for pair: {}/{}, depth: {:?}",
        request.base,
        request.quote,
        request.depth
    );

    let mut results = HashMap::new();

    for (provider_name, client) in clients.providers() {
        log::debug!("Fetching order book from provider: {}", provider_name);

        let result = match client.get_latest_order_book(request.clone()).await {
            Ok(data) => {
                log::info!("Successfully fetched order book from {}", provider_name);
                super::types::ProviderBookResult {
                    success: true,
                    data: Some(data),
                    error: None,
                }
            }
            Err(err) => {
                log::error!("Failed to fetch order book from {}: {}", provider_name, err);
                super::types::ProviderBookResult {
                    success: false,
                    data: None,
                    error: Some(err.to_string()),
                }
            }
        };

        results.insert(provider_name.clone(), result);
    }

    log::info!(
        "Completed fetching order books from all {} providers",
        results.len()
    );
    results
}
