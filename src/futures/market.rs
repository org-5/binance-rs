use std::collections::BTreeMap;

use serde_json::Value;

use crate::api::Futures;
use crate::api::API;
use crate::client::Client;
use crate::config::Config;
use crate::errors::Result;
use crate::futures::model::AggTrades;
use crate::futures::model::LiquidationOrders;
use crate::futures::model::MarkPrices;
use crate::futures::model::OpenInterest;
use crate::futures::model::OpenInterestHist;
use crate::futures::model::OrderBook;
use crate::futures::model::PriceStats;
use crate::futures::model::Trades;
use crate::model::BookTickers;
use crate::model::KlineSummaries;
use crate::model::KlineSummary;
use crate::model::SymbolPrice;
use crate::model::Tickers;
use crate::spot::model::Prices;
use crate::util::build_request;
use crate::util::build_signed_request;

// TODO
// Make enums for Strings
// Add limit parameters to functions
// Implement all functions

#[derive(Clone, Debug)]
pub struct Market {
    pub client: Client,
    pub recv_window: u64,
}

impl Market {
    /// Creates a new Market instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the Client fails to be created.
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Result<Self> {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    /// Creates a new Market instance with a Config.
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

    /// Order book (Default 100; max 1000)
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_depth<S>(&self, symbol: S) -> Result<OrderBook>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);

        self.client
            .get(API::Futures(Futures::Depth), Some(request))
            .await
    }

    /// Order book at a custom depth. Currently supported values
    /// are 5, 10, 20, 50, 100, 500, 1000
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_custom_depth<S>(&self, symbol: S, depth: u64) -> Result<OrderBook>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("limit".into(), depth.to_string());
        let request = build_request(parameters);
        self.client
            .get(API::Futures(Futures::Depth), Some(request))
            .await
    }

    /// Recent trades list
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_trades<S>(&self, symbol: S) -> Result<Trades>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);
        self.client
            .get(API::Futures(Futures::Trades), Some(request))
            .await
    }

    /// Old trade lookup
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_historical_trades<S1, S2, S3>(
        &self,
        symbol: S1,
        from_id: S2,
        limit: S3,
    ) -> Result<Trades>
    where
        S1: Into<String>,
        S2: Into<Option<u64>>,
        S3: Into<Option<u16>>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());

        // Add three optional parameters
        if let Some(lt) = limit.into() {
            parameters.insert("limit".into(), format!("{lt}"));
        }
        if let Some(fi) = from_id.into() {
            parameters.insert("fromId".into(), format!("{fi}"));
        }

        let request = build_signed_request(parameters, self.recv_window)?;

        self.client
            .get_signed(API::Futures(Futures::HistoricalTrades), Some(request))
            .await
    }

    /// Compressed/Aggregate trades list
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_agg_trades<S1, S2, S3, S4, S5>(
        &self,
        symbol: S1,
        from_id: S2,
        start_time: S3,
        end_time: S4,
        limit: S5,
    ) -> Result<AggTrades>
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
            .get(API::Futures(Futures::AggTrades), Some(request))
            .await
    }

    /// Returns up to 'limit' klines for given symbol and interval ("1m", "5m",
    /// ...) [doc](https://github.com/binance-exchange/binance-official-api-docs/blob/master/rest-api.md#klinecandlestick-data)
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
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
            .get(API::Futures(Futures::Klines), Some(request))
            .await?;

        let klines = KlineSummaries::AllKlineSummaries(
            data.iter()
                .map(std::convert::TryInto::try_into)
                .collect::<Result<Vec<KlineSummary>>>()?,
        );

        Ok(klines)
    }

    /// 24hr ticker price change statistics
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_24h_price_stats<S>(&self, symbol: S) -> Result<PriceStats>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);

        self.client
            .get(API::Futures(Futures::Ticker24hr), Some(request))
            .await
    }

    /// 24hr ticker price change statistics for all symbols
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_all_24h_price_stats(&self) -> Result<Vec<PriceStats>> {
        self.client
            .get(API::Futures(Futures::Ticker24hr), None)
            .await
    }

    /// Latest price for ONE symbol.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_price<S>(&self, symbol: S) -> Result<SymbolPrice>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();

        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);

        self.client
            .get(API::Futures(Futures::TickerPrice), Some(request))
            .await
    }

    /// Latest price for all symbols.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_all_prices(&self) -> Result<Prices> {
        self.client
            .get(API::Futures(Futures::TickerPrice), None)
            .await
    }

    /// Symbols order book ticker
    /// -> Best price/qty on the order book for ALL symbols.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_all_book_tickers(&self) -> Result<BookTickers> {
        self.client
            .get(API::Futures(Futures::BookTicker), None)
            .await
    }

    /// Best price/qty on the order book for ONE symbol
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_book_ticker<S>(&self, symbol: S) -> Result<Tickers>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);
        self.client
            .get(API::Futures(Futures::BookTicker), Some(request))
            .await
    }

    /// Mark price and funding rate
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_mark_prices(&self) -> Result<MarkPrices> {
        self.client
            .get(API::Futures(Futures::PremiumIndex), None)
            .await
    }

    /// Get all liquidation orders
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_all_liquidation_orders(&self) -> Result<LiquidationOrders> {
        self.client
            .get(API::Futures(Futures::AllForceOrders), None)
            .await
    }

    /// Get open interest
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn open_interest<S>(&self, symbol: S) -> Result<OpenInterest>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_request(parameters);
        self.client
            .get(API::Futures(Futures::OpenInterest), Some(request))
            .await
    }

    /// Get open interest statistics
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn open_interest_statistics<S1, S2, S3, S4, S5>(
        &self,
        symbol: S1,
        period: S2,
        limit: S3,
        start_time: S4,
        end_time: S5,
    ) -> Result<Vec<OpenInterestHist>>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<Option<u16>>,
        S4: Into<Option<u64>>,
        S5: Into<Option<u64>>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("period".into(), period.into());

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
        self.client
            .get(API::Futures(Futures::OpenInterestHist), Some(request))
            .await
    }
}
