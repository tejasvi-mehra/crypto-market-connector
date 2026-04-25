// Services module containing business logic and provider implementations.
// Handles order book fetching from multiple crypto exchanges.

pub mod binance; // Binance exchange client implementation
pub mod crypto_com; // Crypto.com exchange client implementation
pub mod provider; // Provider client registry and dispatch logic
pub mod types; // Shared data types and error definitions

// Re-exports for convenience
pub use provider::Clients;
pub use types::{OrderBookRequest, OrderBookResponse, ProviderError};
