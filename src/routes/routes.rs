use crate::app::AppState;
use crate::services::{
    provider,
    types::{OrderBookRequest, ProviderBookRequest, ProviderBookResult},
};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};

// Creates and returns the main router for all API routes.
// Routes are defined here and merged into the application router.
pub fn router() -> Router<AppState> {
    log::info!("Setting up API routes");
    Router::new()
        .route("/get_pair_price", post(get_pair_price))
        .route("/get_all_books", post(get_all_books))
}

// Handler for fetching order book from a single provider.
// Accepts an OrderBookRequest and returns the order book data or an error.
async fn get_pair_price(
    State(state): State<AppState>,
    Json(payload): Json<OrderBookRequest>,
) -> Result<Json<crate::services::types::OrderBookResponse>, (StatusCode, String)> {
    log::info!("Received request for single provider order book: {:?}", payload);

    match provider::fetch_order_book(&state.clients, payload).await {
        Ok(response) => {
            log::info!("Successfully fetched order book for provider");
            Ok(Json(response))
        }
        Err(error) => {
            log::error!("Failed to fetch order book: {}", error);
            Err((StatusCode::BAD_REQUEST, error.to_string()))
        }
    }
}

// Handler for fetching order books from all available providers.
// Accepts a ProviderBookRequest and returns a map of provider names to their results.
async fn get_all_books(
    State(state): State<AppState>,
    Json(payload): Json<ProviderBookRequest>,
) -> Json<std::collections::HashMap<String, ProviderBookResult>> {
    log::info!("Received request for all providers order books: {:?}", payload);

    let results = provider::fetch_all_order_books(&state.clients, payload).await;
    log::info!("Fetched order books from {} providers", results.len());

    Json(results)
}
