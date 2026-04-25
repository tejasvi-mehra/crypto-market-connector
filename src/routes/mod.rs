// Routes module containing API endpoint definitions.
// Provides the main router that combines all route handlers.

pub mod routes;

use crate::app::AppState;
use axum::Router;

// Creates and returns the main API router.
// Merges all route modules into a single router with shared application state.
pub fn router() -> Router<AppState> {
    Router::new().merge(routes::router())
}
