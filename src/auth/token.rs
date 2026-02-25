//! Token storage using the system keyring.
//!
//! Provides secure storage for OAuth2 access tokens using the OS keyring.

use keyring::Entry;

use crate::error::{AuthError, Error};

const SERVICE_NAME: &str = "better-ch";
const TOKEN_KEY: &str = "access_token";

/// Token storage using system keyring.
pub struct TokenStore {
    entry: Entry,
}

impl TokenStore {
    /// Create a new token store.
    pub fn new() -> Result<Self, Error> {
        let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)
            .map_err(|e| Error::Auth(AuthError::TokenStorage(e.to_string())))?;
        Ok(Self { entry })
    }

    /// Store the access token.
    pub fn store_token(&self, token: &str) -> Result<(), Error> {
        self.entry
            .set_password(token)
            .map_err(|e| Error::Auth(AuthError::TokenStorage(e.to_string())))?;
        Ok(())
    }

    /// Retrieve the stored access token.
    pub fn get_token(&self) -> Result<Option<String>, Error> {
        match self.entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Error::Auth(AuthError::TokenStorage(e.to_string()))),
        }
    }

    /// Delete the stored token.
    pub fn delete_token(&self) -> Result<(), Error> {
        match self.entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Error::Auth(AuthError::TokenStorage(e.to_string()))),
        }
    }
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new().expect("failed to create token store")
    }
}