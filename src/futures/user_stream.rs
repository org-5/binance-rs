use crate::api::Futures;
use crate::api::API;
use crate::client::Client;
use crate::config::Config;
use crate::errors::Result;
use crate::spot::model::Success;
use crate::spot::model::UserDataStream;

#[derive(Clone)]
pub struct UserStream {
    pub client: Client,
    pub recv_window: u64,
}

impl UserStream {
    /// Create a new `UserStream` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the Client fails to be created.
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Result<Self> {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    /// Create a new `UserStream` instance with a Config.
    ///
    /// # Errors
    ///
    /// Returns an error if the Client fails to be created.
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
            recv_window: config.recv_window,
        })
    }

    /// User Stream
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn start(&self) -> Result<UserDataStream> {
        self.client
            .post(API::Futures(Futures::UserDataStream))
            .await
    }

    /// Keep alive a User Stream
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn keep_alive(&self, listen_key: &str) -> Result<Success> {
        self.client
            .put(API::Futures(Futures::UserDataStream), listen_key)
            .await
    }

    /// Close a User Stream
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn close(&self, listen_key: &str) -> Result<Success> {
        self.client
            .delete(API::Futures(Futures::UserDataStream), listen_key)
            .await
    }
}
