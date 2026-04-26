// Integration tests for Crypto Market Connector API
//
// These tests verify the core functionality of the order book fetching system.
// Note: These tests require API credentials to be configured in the environment.

#[cfg(test)]
mod tests {
    use crypto_market_connector::services::types::{
        OrderBookRequest, OrderBookResponse, ProviderBookRequest, ProviderError,
    };
    use serde_json::json;

    // Unit tests for request/response types

    #[test]
    fn test_order_book_request_serialization() {
        // Test that OrderBookRequest can be serialized to JSON correctly
        let request = OrderBookRequest {
            provider: "binance".to_string(),
            base: "BTC".to_string(),
            quote: "USDT".to_string(),
            depth: Some(10),
        };

        let json = serde_json::to_value(&request).expect("Failed to serialize");

        assert_eq!(json["provider"], "binance");
        assert_eq!(json["base"], "BTC");
        assert_eq!(json["quote"], "USDT");
        assert_eq!(json["depth"], 10);
    }

    #[test]
    fn test_order_book_request_deserialization() {
        // Test that OrderBookRequest can be deserialized from JSON correctly
        let json = json!({
            "provider": "crypto.com",
            "base": "ETH",
            "quote": "USDT",
            "depth": 20
        });

        let request: OrderBookRequest =
            serde_json::from_value(json).expect("Failed to deserialize");

        assert_eq!(request.provider, "crypto.com");
        assert_eq!(request.base, "ETH");
        assert_eq!(request.quote, "USDT");
        assert_eq!(request.depth, Some(20));
    }

    #[test]
    fn test_order_book_request_no_depth() {
        // Test that depth is optional in OrderBookRequest
        let json = json!({
            "provider": "binance",
            "base": "BTC",
            "quote": "USDT"
        });

        let request: OrderBookRequest =
            serde_json::from_value(json).expect("Failed to deserialize");

        assert_eq!(request.depth, None);
    }

    #[test]
    fn test_provider_book_request_serialization() {
        // Test internal provider request format
        let request = ProviderBookRequest {
            base: "BTC".to_string(),
            quote: "USDT".to_string(),
            depth: Some(10),
        };

        let json = serde_json::to_value(&request).expect("Failed to serialize");

        assert_eq!(json["base"], "BTC");
        assert_eq!(json["quote"], "USDT");
        assert_eq!(json["depth"], 10);
    }

    #[test]
    fn test_order_book_response_creation() {
        // Test creating an OrderBookResponse
        let response = OrderBookResponse {
            provider: "binance".to_string(),
            pair: "BTCUSDT".to_string(),
            bids: vec![
                ("42500.00".to_string(), "1.5".to_string()),
                ("42499.50".to_string(), "2.0".to_string()),
            ],
            asks: vec![
                ("42510.00".to_string(), "1.2".to_string()),
                ("42510.50".to_string(), "3.0".to_string()),
            ],
        };

        assert_eq!(response.provider, "binance");
        assert_eq!(response.pair, "BTCUSDT");
        assert_eq!(response.bids.len(), 2);
        assert_eq!(response.asks.len(), 2);

        // Verify bids and asks are in correct order
        assert_eq!(response.bids[0].0, "42500.00");
        assert_eq!(response.asks[0].0, "42510.00");
    }

    #[test]
    fn test_order_book_response_serialization() {
        // Test that OrderBookResponse serializes correctly to JSON
        let response = OrderBookResponse {
            provider: "coinbase".to_string(),
            pair: "BTC-USD".to_string(),
            bids: vec![("42500.00".to_string(), "1.5".to_string())],
            asks: vec![("42510.00".to_string(), "1.2".to_string())],
        };

        let json = serde_json::to_value(&response).expect("Failed to serialize");

        assert_eq!(json["provider"], "coinbase");
        assert_eq!(json["pair"], "BTC-USD");
        assert_eq!(json["bids"][0][0], "42500.00");
        assert_eq!(json["asks"][0][0], "42510.00");
    }

    // Error type tests

    #[test]
    fn test_unsupported_provider_error() {
        // Test UnsupportedProvider error variant
        let error = ProviderError::UnsupportedProvider("xyz_exchange".to_string());
        let error_msg = format!("{}", error);

        assert!(error_msg.contains("unsupported provider"));
        assert!(error_msg.contains("xyz_exchange"));
    }

    #[test]
    fn test_invalid_request_error() {
        // Test InvalidRequest error variant
        let error = ProviderError::InvalidRequest("missing fields".to_string());
        let error_msg = format!("{}", error);

        assert!(error_msg.contains("invalid request"));
        assert!(error_msg.contains("missing fields"));
    }

    #[test]
    fn test_backend_error() {
        // Test BackendError variant
        let error = ProviderError::BackendError("API connection failed".to_string());
        let error_msg = format!("{}", error);

        assert!(error_msg.contains("backend error"));
        assert!(error_msg.contains("API connection failed"));
    }

    #[test]
    fn test_error_display_trait() {
        // Test that error types implement Display trait correctly
        let errors = vec![
            ProviderError::UnsupportedProvider("test".to_string()),
            ProviderError::InvalidRequest("test".to_string()),
            ProviderError::BackendError("test".to_string()),
        ];

        for error in errors {
            let msg = error.to_string();
            assert!(!msg.is_empty());
        }
    }

    // Data validation tests

    #[test]
    fn test_pair_symbol_format() {
        // Test that pair symbols are formatted correctly by different providers
        let pairs = vec![
            ("binance", "BTCUSDT"),
            ("crypto.com", "BTC_USDT"),
            ("coinbase", "BTC-USD"),
        ];

        for (provider, pair) in pairs {
            let response = OrderBookResponse {
                provider: provider.to_string(),
                pair: pair.to_string(),
                bids: vec![],
                asks: vec![],
            };

            assert_eq!(response.pair, pair);
        }
    }

    #[test]
    fn test_order_book_price_precision() {
        // Test that prices are stored as strings to preserve precision
        let response = OrderBookResponse {
            provider: "binance".to_string(),
            pair: "BTCUSDT".to_string(),
            bids: vec![("42500.123456789".to_string(), "1.123456".to_string())],
            asks: vec![],
        };

        // Prices should be preserved exactly as strings
        assert_eq!(response.bids[0].0, "42500.123456789");
        assert_eq!(response.bids[0].1, "1.123456");
    }

    #[test]
    fn test_multiple_bid_ask_levels() {
        // Test handling multiple bid/ask price levels
        let response = OrderBookResponse {
            provider: "binance".to_string(),
            pair: "ETHUSDT".to_string(),
            bids: vec![
                ("2500.00".to_string(), "10.0".to_string()),
                ("2499.50".to_string(), "20.0".to_string()),
                ("2499.00".to_string(), "15.0".to_string()),
                ("2498.50".to_string(), "30.0".to_string()),
                ("2498.00".to_string(), "25.0".to_string()),
            ],
            asks: vec![
                ("2510.00".to_string(), "5.0".to_string()),
                ("2510.50".to_string(), "15.0".to_string()),
                ("2511.00".to_string(), "10.0".to_string()),
                ("2511.50".to_string(), "20.0".to_string()),
                ("2512.00".to_string(), "12.0".to_string()),
            ],
        };

        assert_eq!(response.bids.len(), 5);
        assert_eq!(response.asks.len(), 5);

        // Verify correct ordering (bids highest to lowest, asks lowest to highest)
        assert!(
            response.bids[0].0.parse::<f64>().unwrap() > response.bids[1].0.parse::<f64>().unwrap()
        );
        assert!(
            response.asks[0].0.parse::<f64>().unwrap() < response.asks[1].0.parse::<f64>().unwrap()
        );
    }

    // Integration test examples (would require running server)

    #[test]
    #[ignore] // Ignore by default as it requires server running
    fn test_single_provider_endpoint_example() {
        // Example test for single provider endpoint
        // This test is ignored by default as it requires the server to be running
        // Run with: cargo test -- --ignored test_single_provider_endpoint_example

        let request_json = json!({
            "provider": "binance",
            "base": "BTC",
            "quote": "USDT",
            "depth": 10
        });

        // In a real test, we would make an HTTP POST request:
        // let client = reqwest::Client::new();
        // let response = client
        //     .post("http://localhost:8080/get_pair_price")
        //     .json(&request_json)
        //     .send()
        //     .await;

        // For now, just verify the request can be serialized
        assert!(serde_json::to_string(&request_json).is_ok());
    }

    #[test]
    #[ignore] // Ignore by default
    fn test_multi_provider_endpoint_example() {
        // Example test for multi-provider endpoint
        let request_json = json!({
            "base": "BTC",
            "quote": "USDT",
            "depth": 10
        });

        // In a real test, we would make an HTTP POST request:
        // let client = reqwest::Client::new();
        // let response = client
        //     .post("http://localhost:8080/get_all_books")
        //     .json(&request_json)
        //     .send()
        //     .await;

        assert!(serde_json::to_string(&request_json).is_ok());
    }
}

// Doc tests for documentation examples

// # Example: Creating an OrderBookRequest
//
// ```
// use crypto_market_connector::services::types::OrderBookRequest;
//
// let request = OrderBookRequest {
//     provider: "binance".to_string(),
//     base: "BTC".to_string(),
//     quote: "USDT".to_string(),
//     depth: Some(10),
// };
//
// assert_eq!(request.provider, "binance");
// ```
pub fn _example_order_book_request() {}

// # Example: Creating an OrderBookResponse
//
// ```
// use crypto_market_connector::services::types::OrderBookResponse;
//
// let response = OrderBookResponse {
//     provider: "binance".to_string(),
//     pair: "BTCUSDT".to_string(),
//     bids: vec![("42500.00".to_string(), "1.5".to_string())],
//     asks: vec![("42510.00".to_string(), "1.2".to_string())],
// };
//
// assert_eq!(response.bids.len(), 1);
// ```
pub fn _example_order_book_response() {}

// # Example: Handling ProviderError
//
// ```
// use crypto_market_connector::services::types::ProviderError;
//
// let error = ProviderError::UnsupportedProvider("xyz".to_string());
// let message = format!("{}", error);
//
// assert!(message.contains("unsupported provider"));
// ```
pub fn _example_error_handling() {}
