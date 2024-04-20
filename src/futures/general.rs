use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use error_chain::bail;

use crate::api::Futures;
use crate::api::API;
use crate::client::Client;
use crate::config::Config;
use crate::errors::Result;
use crate::futures::model::ExchangeInformation;
use crate::futures::model::Symbol;
use crate::model::ServerTime;

const CACHE_TTL: u64 = 600; // 10 minutes.

#[derive(Clone, Debug)]
pub struct General {
    pub client: Client,
    pub(crate) cache: Option<ExchangeInformation>,
    pub(crate) last_update: Option<u64>,
}

impl General {
    /// Create a new General instance.
    /// If `api_key` an `secret_key` are provided, the client will be
    /// authenticated.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created.    
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Result<Self> {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    /// Create a new General instance with a configuration.
    /// If `api_key` an `secret_key` are provided, the client will be
    /// authenticated.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created.    
    pub fn new_with_config(
        api_key: Option<String>,
        secret_key: Option<String>,
        config: &Config,
    ) -> Result<Self> {
        Ok(Self {
            client: Client::new(
                api_key,
                secret_key,
                config.futures_rest_api_endpoint.clone(),
            )?,
            cache: None,
            last_update: None,
        })
    }

    /// Test connectivity
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn ping(&self) -> Result<String> {
        self.client.get(API::Futures(Futures::Ping), None).await?;
        Ok("pong".into())
    }

    /// Check server time
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_server_time(&self) -> Result<ServerTime> {
        self.client.get(API::Futures(Futures::Time), None).await
    }

    /// Obtain exchange information
    /// - Current exchange trading rules and symbol information
    /// The boolean is true if the cache was used.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache is empty.
    pub fn exchange_info(&self) -> Result<(ExchangeInformation, bool)> {
        if self.has_cache() {
            let Some(cache) = self.cache.clone() else {
                unreachable!("`has_cache` checks if that this is not None.")
            };
            Ok((cache, true))
        } else {
            Err("No cache".into())
        }
    }

    /// Update the cache with the latest exchange information.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn update_cache(&mut self) -> Result<()> {
        let info: ExchangeInformation = self
            .client
            .get(API::Futures(Futures::ExchangeInfo), None)
            .await?;
        self.cache = Some(info.clone());
        self.last_update = Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
        Ok(())
    }

    /// Check if the cache is still valid.
    ///
    /// # Returns
    ///
    /// Returns true if the cache is still valid.
    ///
    /// # Panics
    ///
    /// Panics if the system time cannot be retrieved.
    #[must_use]
    pub fn has_cache(&self) -> bool {
        self.cache.is_some()
            && self.last_update.is_some()
            && SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - self.last_update.unwrap()
                < CACHE_TTL
    }

    /// Get Symbol information
    ///
    /// # Errors
    ///
    /// Returns an error if the symbol is not found.
    pub fn get_symbol_info<S>(&mut self, symbol: S) -> Result<Symbol>
    where
        S: Into<String>,
    {
        let upper_symbol = symbol.into().to_uppercase();
        match self.exchange_info() {
            Ok(info) => {
                for item in info.0.symbols {
                    if item.symbol == upper_symbol {
                        return Ok(item);
                    }
                }
                bail!("Symbol not found")
            }
            Err(e) => Err(e),
        }
    }
}
