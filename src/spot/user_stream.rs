use super::model::Success;
use super::model::UserDataStream;
use crate::api::Spot;
use crate::api::API;
use crate::client::Client;
use crate::config::Config;
use crate::errors::Result;

#[derive(Clone)]
pub struct UserStream {
    pub client: Client,
    pub recv_window: u64,
}

impl UserStream {
    /// Initialize a new `UserStream` instance
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be initialized.
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Result<Self> {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    /// Initialize a new `UserStream` instance with a configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be initialized.
    pub fn new_with_config(
        api_key: Option<String>,
        secret_key: Option<String>,
        config: &Config,
    ) -> Result<Self> {
        Ok(Self {
            client: Client::new(api_key, secret_key, config.rest_api_endpoint.clone())?,
            recv_window: config.recv_window,
        })
    }

    /// User Stream
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn start(&self) -> Result<UserDataStream> {
        self.client.post(API::Spot(Spot::UserDataStream)).await
    }

    /// Current open orders on a symbol
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn keep_alive(&self, listen_key: &str) -> Result<Success> {
        self.client
            .put(API::Spot(Spot::UserDataStream), listen_key)
            .await
    }

    /// Current open orders on a symbol
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn close(&self, listen_key: &str) -> Result<Success> {
        self.client
            .delete(API::Spot(Spot::UserDataStream), listen_key)
            .await
    }
}
