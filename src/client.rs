use bytes::Bytes;
use error_chain::bail;
use hex::encode as hex_encode;
use hmac::Hmac;
use hmac::Mac;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::USER_AGENT;
use reqwest::Response;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use sha2::Sha256;
use tracing::debug;

use crate::api::API;
use crate::errors::BinanceContentError;
use crate::errors::ErrorKind;
use crate::errors::Result;

#[derive(Clone, Debug)]
pub struct Client {
    api_key: String,
    secret_key: String,
    host: String,
    inner: reqwest::Client,
}

impl Client {
    pub fn new(api_key: Option<String>, secret_key: Option<String>, host: String) -> Result<Self> {
        Ok(Client {
            api_key: api_key.unwrap_or_default(),
            secret_key: secret_key.unwrap_or_default(),
            host,
            inner: reqwest::Client::builder().pool_idle_timeout(None).build()?,
        })
    }

    pub async fn get_signed<T: DeserializeOwned>(
        &self,
        endpoint: API,
        request: Option<String>,
    ) -> Result<T> {
        let url = self.sign_request(endpoint, request);
        let client = &self.inner;
        let response = client
            .get(url.as_str())
            .headers(self.build_headers(true)?)
            .send()
            .await?;

        self.handler(response).await
    }

    pub async fn get_signed_bytes(&self, endpoint: API, request: Option<String>) -> Result<Bytes> {
        let url = self.sign_request(endpoint, request);
        let client = &self.inner;
        let response = client
            .get(url.as_str())
            .headers(self.build_headers(true)?)
            .send()
            .await?;

        self.bytes_handler(response).await
    }

    pub async fn post_signed<T: DeserializeOwned>(
        &self,
        endpoint: API,
        request: String,
    ) -> Result<T> {
        let url = self.sign_request(endpoint, Some(request));
        let client = &self.inner;
        let response = client
            .post(url.as_str())
            .headers(self.build_headers(true)?)
            .send()
            .await?;

        self.handler(response).await
    }

    pub async fn delete_signed<T: DeserializeOwned>(
        &self,
        endpoint: API,
        request: Option<String>,
    ) -> Result<T> {
        let url = self.sign_request(endpoint, request);
        let client = &self.inner;
        let response = client
            .delete(url.as_str())
            .headers(self.build_headers(true)?)
            .send()
            .await?;

        self.handler(response).await
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: API,
        request: Option<String>,
    ) -> Result<T> {
        let mut url: String = format!("{}{}", self.host, String::from(endpoint));
        if let Some(request) = request {
            if !request.is_empty() {
                url.push_str(format!("?{request}").as_str());
            }
        }

        let client = &self.inner;
        let response = client.get(url.as_str()).send().await?;

        self.handler(response).await
    }

    pub async fn post<T: DeserializeOwned>(&self, endpoint: API) -> Result<T> {
        let url: String = format!("{}{}", self.host, String::from(endpoint));

        let client = &self.inner;
        let response = client
            .post(url.as_str())
            .headers(self.build_headers(false)?)
            .send()
            .await?;

        self.handler(response).await
    }

    pub async fn put<T: DeserializeOwned>(&self, endpoint: API, listen_key: &str) -> Result<T> {
        let url: String = format!("{}{}", self.host, String::from(endpoint));
        let data: String = format!("listenKey={listen_key}");

        let client = &self.inner;
        let response = client
            .put(url.as_str())
            .headers(self.build_headers(false)?)
            .body(data)
            .send()
            .await?;

        self.handler(response).await
    }

    pub async fn delete<T: DeserializeOwned>(&self, endpoint: API, listen_key: &str) -> Result<T> {
        let url: String = format!("{}{}", self.host, String::from(endpoint));
        let data: String = format!("listenKey={listen_key}");

        let client = &self.inner;
        let response = client
            .delete(url.as_str())
            .headers(self.build_headers(false)?)
            .body(data)
            .send()
            .await?;

        self.handler(response).await
    }

    // Request must be signed
    fn sign_request(&self, endpoint: API, request: Option<String>) -> String {
        // If endpoint starts with http, then it is a full url, no need to add host.
        let host = if String::from(endpoint.clone()).starts_with("http") {
            String::new()
        } else {
            self.host.clone()
        };
        if let Some(request) = request {
            let mut signed_key =
                Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes()).unwrap();
            signed_key.update(request.as_bytes());
            let signature = hex_encode(signed_key.finalize().into_bytes());
            let request_body: String = format!("{request}&signature={signature}");
            format!("{}{}?{}", host, String::from(endpoint), request_body)
        } else {
            let signed_key = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes()).unwrap();
            let signature = hex_encode(signed_key.finalize().into_bytes());
            let request_body: String = format!("&signature={signature}");
            format!("{}{}?{}", host, String::from(endpoint), request_body)
        }
    }

    fn build_headers(&self, content_type: bool) -> Result<HeaderMap> {
        let mut custom_headers = HeaderMap::new();

        custom_headers.insert(USER_AGENT, HeaderValue::from_static("binance-rs"));
        if content_type {
            custom_headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        }
        custom_headers.insert(
            HeaderName::from_static("x-mbx-apikey"),
            HeaderValue::from_str(self.api_key.as_str())?,
        );

        Ok(custom_headers)
    }

    async fn bytes_handler(&self, response: Response) -> Result<Bytes> {
        if response.headers().contains_key("x-mbx-used-weight-1m") {
            let used_weights = response.headers().get("x-mbx-used-weight-1m").unwrap();
            debug!("Used weights: {}", used_weights.to_str().unwrap());
        }

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            bail!(ErrorKind::TooManyRequest)
        }
        assert!(
            response.status() != StatusCode::IM_A_TEAPOT,
            "We were told we are a teapot"
        );

        match response.status() {
            StatusCode::OK => Ok(response.bytes().await?),
            StatusCode::INTERNAL_SERVER_ERROR => {
                bail!("Internal Server Error");
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                bail!("Service Unavailable");
            }
            StatusCode::UNAUTHORIZED => {
                bail!("Unauthorized");
            }
            StatusCode::BAD_REQUEST => {
                let error: BinanceContentError = response.json().await?;

                Err(ErrorKind::BinanceError(error).into())
            }
            s => {
                bail!(format!("Received response: {:#?}", s));
            }
        }
    }

    async fn handler<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        if response.headers().contains_key("x-mbx-used-weight-1m") {
            let used_weights = response.headers().get("x-mbx-used-weight-1m").unwrap();
            debug!("Used weights: {}", used_weights.to_str().unwrap());
        }

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            bail!(ErrorKind::TooManyRequest)
        }
        assert!(
            response.status() != StatusCode::IM_A_TEAPOT,
            "We were told we are a teapot"
        );

        match response.status() {
            StatusCode::OK => Ok(response.json::<T>().await?),
            StatusCode::INTERNAL_SERVER_ERROR => {
                bail!("Internal Server Error");
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                bail!("Service Unavailable");
            }
            StatusCode::UNAUTHORIZED => {
                bail!("Unauthorized");
            }
            StatusCode::BAD_REQUEST => {
                let error: BinanceContentError = response.json().await?;

                Err(ErrorKind::BinanceError(error).into())
            }
            s => {
                bail!(format!("Received response: {:?}", s));
            }
        }
    }
}
