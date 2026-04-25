use serde::{Deserialize, Serialize};

// Request structure for fetching order book from a specific provider.
// Contains provider name and trading pair information.
#[derive(Clone, Debug, Deserialize)]
pub struct OrderBookRequest {
    pub provider: String,
    pub base: String,
    pub quote: String,
    pub depth: Option<u32>,
}

// Response structure containing order book data from a provider.
// Includes bids and asks as vectors of (price, quantity) tuples.
#[derive(Debug, Serialize)]
pub struct OrderBookResponse {
    pub provider: String,
    pub pair: String,
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

// Request structure for provider-specific order book fetching.
// Used internally when dispatching to individual provider clients.
#[derive(Clone, Debug, Deserialize)]
pub struct ProviderBookRequest {
    pub base: String,
    pub quote: String,
    pub depth: Option<u32>,
}

// Result structure for multi-provider order book requests.
// Contains success status, optional data, and optional error message.
#[derive(Debug, Serialize)]
pub struct ProviderBookResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<OrderBookResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// Enumeration of possible provider-related errors.
// Implements Display and Error traits for proper error handling.
#[derive(Debug)]
pub enum ProviderError {
    // The requested provider is not supported by the system.
    UnsupportedProvider(String),
    // The request parameters are invalid.
    InvalidRequest(String),
    // An error occurred while communicating with the provider's backend.
    BackendError(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::UnsupportedProvider(provider) => {
                write!(f, "unsupported provider: {}", provider)
            }
            ProviderError::InvalidRequest(message) => write!(f, "invalid request: {}", message),
            ProviderError::BackendError(message) => write!(f, "backend error: {}", message),
        }
    }
}

impl std::error::Error for ProviderError {}
