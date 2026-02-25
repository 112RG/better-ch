//! Authentication module for Anypoint Platform.
//!
//! Provides OAuth2 client-credentials authentication and secure token storage.

pub mod oauth;
pub mod token;

pub use oauth::Authenticator;
pub use token::TokenStore;