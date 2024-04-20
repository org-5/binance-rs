use std::collections::BTreeMap;

use serde_json::Value;

use super::model::AggTrade;
use super::model::AveragePrice;
use super::model::BookTickers;
use super::model::KlineSummaries;
use super::model::KlineSummary;
use super::model::OrderBook;
use super::model::PriceStats;
use super::model::Prices;
use super::model::SymbolPrice;
use super::model::Tickers;
use crate::api::Spot;
use crate::api::API;
use crate::client::Client;
use crate::config::Config;
use crate::errors::Result;
use crate::util::build_request;

#[derive(Clone, Debug)]
pub struct Market {
    pub client: Client,
    pub recv_window: u64,
}

// Market Data endpoints
impl Market {
    /// Initialize a new Market instance
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be initialized
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Result<Self> {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    /// Initialize a new Market instance with a configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be initialized
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

    /// Order book at the default depth of 100
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_depth<S>(&self, symbol: S) -> Result<OrderBook>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);
        self.client.get(API::Spot(Spot::Depth), Some(request)).await
    }

    /// Order book at a custom depth. Currently supported values
    /// are 5, 10, 20, 50, 100, 500, 1000 and 5000
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_custom_depth<S>(&self, symbol: S, depth: u64) -> Result<OrderBook>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("limit".into(), depth.to_string());
        let request = build_request(parameters);
        self.client.get(API::Spot(Spot::Depth), Some(request)).await
    }

    /// Latest price for ALL symbols.
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_all_prices(&self) -> Result<Prices> {
        self.client.get(API::Spot(Spot::Price), None).await
    }

    /// Latest price for ONE symbol.
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_price<S>(&self, symbol: S) -> Result<SymbolPrice>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);
        self.client.get(API::Spot(Spot::Price), Some(request)).await
    }

    /// Average price for ONE symbol.
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_average_price<S>(&self, symbol: S) -> Result<AveragePrice>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);
        self.client
            .get(API::Spot(Spot::AvgPrice), Some(request))
            .await
    }

    /// Symbols order book ticker
    /// -> Best price/qty on the order book for ALL symbols.
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_all_book_tickers(&self) -> Result<BookTickers> {
        self.client.get(API::Spot(Spot::BookTicker), None).await
    }

    /// -> Best price/qty on the order book for ONE symbol
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_book_ticker<S>(&self, symbol: S) -> Result<Tickers>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);
        self.client
            .get(API::Spot(Spot::BookTicker), Some(request))
            .await
    }

    /// 24hr ticker price change statistics
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_24h_price_stats<S>(&self, symbol: S) -> Result<PriceStats>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);
        self.client
            .get(API::Spot(Spot::Ticker24hr), Some(request))
            .await
    }

    /// 24hr ticker price change statistics for all symbols
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_all_24h_price_stats(&self) -> Result<Vec<PriceStats>> {
        self.client.get(API::Spot(Spot::Ticker24hr), None).await
    }

    /// Get aggregated historical trades.
    ///
    /// If you provide `start_time`, you also need to provide `end_time`.
    /// If `from_id`, `start_time` and `end_time` are omitted, the most recent
    /// trades are fetched.
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_agg_trades<S1, S2, S3, S4, S5>(
        &self,
        symbol: S1,
        from_id: S2,
        start_time: S3,
        end_time: S4,
        limit: S5,
    ) -> Result<Vec<AggTrade>>
    where
        S1: Into<String>,
        S2: Into<Option<u64>>,
        S3: Into<Option<u64>>,
        S4: Into<Option<u64>>,
        S5: Into<Option<u16>>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());

        // Add three optional parameters
        if let Some(lt) = limit.into() {
            parameters.insert("limit".into(), format!("{lt}"));
        }
        if let Some(st) = start_time.into() {
            parameters.insert("startTime".into(), format!("{st}"));
        }
        if let Some(et) = end_time.into() {
            parameters.insert("endTime".into(), format!("{et}"));
        }
        if let Some(fi) = from_id.into() {
            parameters.insert("fromId".into(), format!("{fi}"));
        }

        let request = build_request(parameters);

        self.client
            .get(API::Spot(Spot::AggTrades), Some(request))
            .await
    }

    /// Returns up to 'limit' klines for given symbol and interval ("1m", "5m",
    /// ...) [docs](https://github.com/binance-exchange/binance-official-api-docs/blob/master/rest-api.md#klinecandlestick-data)
    ///
    /// # Errors
    ///
    /// Returns an error if the request does not succeed.
    pub async fn get_klines<S1, S2, S3, S4, S5>(
        &self,
        symbol: S1,
        interval: S2,
        limit: S3,
        start_time: S4,
        end_time: S5,
    ) -> Result<KlineSummaries>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<Option<u16>>,
        S4: Into<Option<u64>>,
        S5: Into<Option<u64>>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("interval".into(), interval.into());

        // Add three optional parameters
        if let Some(lt) = limit.into() {
            parameters.insert("limit".into(), format!("{lt}"));
        }
        if let Some(st) = start_time.into() {
            parameters.insert("startTime".into(), format!("{st}"));
        }
        if let Some(et) = end_time.into() {
            parameters.insert("endTime".into(), format!("{et}"));
        }

        let request = build_request(parameters);
        let data: Vec<Vec<Value>> = self
            .client
            .get(API::Spot(Spot::Klines), Some(request))
            .await?;

        let klines = KlineSummaries::AllKlineSummaries(
            data.iter()
                .map(std::convert::TryInto::try_into)
                .collect::<Result<Vec<KlineSummary>>>()?,
        );

        Ok(klines)
    }
}
