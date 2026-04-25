// Main library module for the Crypto Market Connector.
// This crate provides a multi-provider crypto order book API service.
// Public modules
pub mod app; // Application setup and server initialization
pub mod middleware; // HTTP middleware for request logging
pub mod routes; // API route definitions and handlers
pub mod services; // Business logic and provider implementations
