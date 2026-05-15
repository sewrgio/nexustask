pub mod domain;
pub mod ports;
pub mod application;
pub mod infrastructure;
pub mod state;

// Re-export commonly used items for tests
pub use infrastructure::database::auth_provider;
pub use ports::auth::AuthProvider;
