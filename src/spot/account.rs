use std::collections::BTreeMap;
use std::fmt::Display;

use error_chain::bail;

use super::model::AccountInformation;
use super::model::Balance;
use super::model::Order;
use super::model::OrderCanceled;
use super::model::TradeHistory;
use super::model::Transaction;
use crate::api::Spot;
use crate::api::API;
use crate::client::Client;
use crate::config::Config;
use crate::errors::Result;
use crate::model::CommissionRates;
use crate::model::Empty;
use crate::util::build_signed_request;

#[derive(Clone)]
pub struct Account {
    pub client: Client,
    pub recv_window: u64,
}

struct OrderRequest {
    pub symbol: String,
    pub qty: f64,
    pub price: f64,
    pub stop_price: Option<f64>,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub new_client_order_id: Option<String>,
}

struct OrderQuoteQuantityRequest {
    pub symbol: String,
    pub quote_order_qty: f64,
    pub price: f64,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub new_client_order_id: Option<String>,
}

pub enum OrderType {
    Limit,
    Market,
    StopLossLimit,
}

impl Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Limit => write!(f, "LIMIT"),
            Self::Market => write!(f, "MARKET"),
            Self::StopLossLimit => write!(f, "STOP_LOSS_LIMIT"),
        }
    }
}

pub enum OrderSide {
    Buy,
    Sell,
}

impl Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buy => write!(f, "BUY"),
            Self::Sell => write!(f, "SELL"),
        }
    }
}

#[allow(clippy::all)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
}

impl Display for TimeInForce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GTC => write!(f, "GTC"),
            Self::IOC => write!(f, "IOC"),
            Self::FOK => write!(f, "FOK"),
        }
    }
}

impl Account {
    /// Create a new Account instance.
    /// If `api_key` an`secret_key` are provided, the client will be authenticated.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created.
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Result<Self> {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    /// Create a new Account instance with a configuration.
    /// If `api_key` an `secret_key` are provided, the client will be authenticated.
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
            client: Client::new(api_key, secret_key, config.rest_api_endpoint.clone())?,
            recv_window: config.recv_window,
        })
    }
    /// Retrieves the account information.
    ///
    /// # Errors
    ///
    /// Returns an error if the account information cannot be retrieved.
    pub async fn get_account(&self) -> Result<AccountInformation> {
        let request = build_signed_request(BTreeMap::new(), self.recv_window)?;
        self.client
            .get_signed(API::Spot(Spot::Account), Some(request))
            .await
    }

    /// Retrieves the comission rates for a symbol
    ///
    /// # Errors
    ///
    /// Returns an error if the commission rates cannot be retrieved.
    pub async fn get_commission<S>(&self, symbol: S) -> Result<CommissionRates>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Spot(Spot::AccountCommission), Some(request))
            .await
    }

    // Balance for a single Asset
    /// Retrieves the balance for a single asset.
    ///
    /// # Errors
    ///
    /// Returns an error if the balance for the asset cannot be retrieved.
    pub async fn get_balance<S>(&self, asset: S) -> Result<Balance>
    where
        S: Into<String>,
    {
        match self.get_account().await {
            Ok(account) => {
                let cmp_asset = asset.into();
                for balance in account.balances {
                    if balance.asset == cmp_asset {
                        return Ok(balance);
                    }
                }
                bail!("Asset not found");
            }
            Err(e) => Err(e),
        }
    }

    /// Retrieves the current open orders for a single symbol.
    ///
    /// # Errors
    ///
    /// Returns an error if the open orders for the symbol cannot be retrieved.
    pub async fn get_open_orders<S>(&self, symbol: S) -> Result<Vec<Order>>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Spot(Spot::OpenOrders), Some(request))
            .await
    }

    /// Retrieves all open orders.
    ///
    /// # Errors
    ///
    /// Returns an error if the open orders cannot be retrieved.
    pub async fn get_all_open_orders(&self) -> Result<Vec<Order>> {
        let parameters: BTreeMap<String, String> = BTreeMap::new();

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Spot(Spot::OpenOrders), Some(request))
            .await
    }

    /// Cancels all open orders for a single symbol.
    ///
    /// # Errors
    ///
    /// Returns an error if the open orders cannot be canceled.
    pub async fn cancel_all_open_orders<S>(&self, symbol: S) -> Result<Vec<OrderCanceled>>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .delete_signed(API::Spot(Spot::OpenOrders), Some(request))
            .await
    }

    /// Retrieves the status of an order.
    ///
    /// # Errors
    ///
    /// Returns an error if the order status cannot be retrieved.
    pub async fn order_status<S>(&self, symbol: S, order_id: u64) -> Result<Order>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("orderId".into(), order_id.to_string());

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Spot(Spot::Order), Some(request))
            .await
    }

    /// Place a test status order
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the test order status cannot be retrieved.
    pub async fn test_order_status<S>(&self, symbol: S, order_id: u64) -> Result<()>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("orderId".into(), order_id.to_string());

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed::<Empty>(API::Spot(Spot::OrderTest), Some(request))
            .await
            .map(|_| ())
    }

    /// Place a limit buy order.
    ///
    /// # Errors
    ///
    /// Returns an error if the limit buy order cannot be placed.
    pub async fn limit_buy<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: None,
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test limit buy order.
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the test limit buy order cannot be placed.
    pub async fn test_limit_buy<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: None,
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    /// Place a limit sell order.
    ///
    /// # Errors
    ///
    /// Returns an error if the limit sell order cannot be placed.
    pub async fn limit_sell<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: None,
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test limit sell order.
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the test limit sell order cannot be placed.
    pub async fn test_limit_sell<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: None,
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    /// Place a market buy order.
    ///
    /// # Errors
    ///
    /// Returns an error if the market buy order cannot be placed.
    pub async fn market_buy<S, F>(&self, symbol: S, qty: F) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price: 0.0,
            stop_price: None,
            order_side: OrderSide::Buy,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test market buy order.
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the test market buy order cannot be placed.
    pub async fn test_market_buy<S, F>(&self, symbol: S, qty: F) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price: 0.0,
            stop_price: None,
            order_side: OrderSide::Buy,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    /// Place a market buy order with quote quantity.
    ///
    /// # Errors
    ///
    /// Returns an error if the market buy order with quote quantity cannot be placed.
    pub async fn market_buy_using_quote_quantity<S, F>(
        &self,
        symbol: S,
        quote_order_qty: F,
    ) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy = OrderQuoteQuantityRequest {
            symbol: symbol.into(),
            quote_order_qty: quote_order_qty.into(),
            price: 0.0,
            order_side: OrderSide::Buy,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_quote_quantity_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test MARKET order with quote quantity - BUY
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the test market buy order with quote quantity cannot be placed.
    pub async fn test_market_buy_using_quote_quantity<S, F>(
        &self,
        symbol: S,
        quote_order_qty: F,
    ) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let buy = OrderQuoteQuantityRequest {
            symbol: symbol.into(),
            quote_order_qty: quote_order_qty.into(),
            price: 0.0,
            order_side: OrderSide::Buy,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_quote_quantity_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    /// Place a market sell order.
    ///
    /// # Errors
    ///
    /// Returns an error if the market sell order cannot be placed.
    pub async fn market_sell<S, F>(&self, symbol: S, qty: F) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price: 0.0,
            stop_price: None,
            order_side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test MARKET order - SELL
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the test market sell order cannot be placed.
    pub async fn test_market_sell<S, F>(&self, symbol: S, qty: F) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price: 0.0,
            stop_price: None,
            order_side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    /// Place a sell market order with quote quantity.
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the market sell order with quote quantity cannot be placed.
    pub async fn market_sell_using_quote_quantity<S, F>(
        &self,
        symbol: S,
        quote_order_qty: F,
    ) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderQuoteQuantityRequest {
            symbol: symbol.into(),
            quote_order_qty: quote_order_qty.into(),
            price: 0.0,
            order_side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_quote_quantity_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test MARKET order with quote quantity - SELL
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the test market sell order with quote quantity cannot be placed.
    pub async fn test_market_sell_using_quote_quantity<S, F>(
        &self,
        symbol: S,
        quote_order_qty: F,
    ) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderQuoteQuantityRequest {
            symbol: symbol.into(),
            quote_order_qty: quote_order_qty.into(),
            price: 0.0,
            order_side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = build_quote_quantity_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    /// Create a stop limit buy order for the given symbol, price and stop
    /// price. Returning a `Transaction` value with the same parameters sent
    /// on the order.
    ///
    ///```no_run
    /// use binance::api::Binance;
    /// use binance::spot::account::*;
    ///
    /// fn main() {
    ///     let api_key = Some("api_key".into());
    ///     let secret_key = Some("secret_key".into());
    ///     let account =  Account::new(api_key, secret_key).unwrap();
    ///     let result = account.stop_limit_buy_order("LTCBTC", 1, 0.1, 0.09, TimeInForce::GTC);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the stop limit buy order cannot be placed.
    pub async fn stop_limit_buy_order<S, F>(
        &self,
        symbol: S,
        qty: F,
        price: f64,
        stop_price: f64,
        time_in_force: TimeInForce,
    ) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: Some(stop_price),
            order_side: OrderSide::Buy,
            order_type: OrderType::StopLossLimit,
            time_in_force,
            new_client_order_id: None,
        };
        let order = build_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Create a stop limit buy test order for the given symbol, price and stop
    /// price. Returning a `Transaction` value with the same parameters sent
    /// on the order.
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    ///```no_run
    /// use binance::api::Binance;
    /// use binance::spot::account::*;
    ///
    /// fn main() {
    ///     let api_key = Some("api_key".into());
    ///     let secret_key = Some("secret_key".into());
    ///     let account =  Account::new(api_key, secret_key).unwrap();
    ///     let result = account.test_stop_limit_buy_order("LTCBTC", 1, 0.1, 0.09, TimeInForce::GTC);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the test stop limit buy order cannot be placed.
    pub async fn test_stop_limit_buy_order<S, F>(
        &self,
        symbol: S,
        qty: F,
        price: f64,
        stop_price: f64,
        time_in_force: TimeInForce,
    ) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: Some(stop_price),
            order_side: OrderSide::Buy,
            order_type: OrderType::StopLossLimit,
            time_in_force,
            new_client_order_id: None,
        };
        let order = build_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    /// Create a stop limit sell order for the given symbol, price and stop
    /// price. Returning a `Transaction` value with the same parameters sent
    /// on the order.
    ///
    ///```no_run
    /// use binance::api::Binance;
    /// use binance::spot::account::*;
    ///
    /// fn main() {
    ///     let api_key = Some("api_key".into());
    ///     let secret_key = Some("secret_key".into());
    ///     let account =  Account::new(api_key, secret_key).unwrap();
    ///     let result = account.stop_limit_sell_order("LTCBTC", 1, 0.1, 0.09, TimeInForce::GTC);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the stop limit sell order cannot be placed.
    pub async fn stop_limit_sell_order<S, F>(
        &self,
        symbol: S,
        qty: F,
        price: f64,
        stop_price: f64,
        time_in_force: TimeInForce,
    ) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: Some(stop_price),
            order_side: OrderSide::Sell,
            order_type: OrderType::StopLossLimit,
            time_in_force,
            new_client_order_id: None,
        };
        let order = build_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Create a stop limit sell order for the given symbol, price and stop
    /// price. Returning a `Transaction` value with the same parameters sent
    /// on the order.
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    ///```no_run
    /// use binance::api::Binance;
    /// use binance::spot::account::*;
    ///
    /// fn main() {
    ///     let api_key = Some("api_key".into());
    ///     let secret_key = Some("secret_key".into());
    ///     let account =  Account::new(api_key, secret_key).unwrap();
    ///     let result = account.test_stop_limit_sell_order("LTCBTC", 1, 0.1, 0.09, TimeInForce::GTC);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the test stop limit sell order cannot be placed.
    pub async fn test_stop_limit_sell_order<S, F>(
        &self,
        symbol: S,
        qty: F,
        price: f64,
        stop_price: f64,
        time_in_force: TimeInForce,
    ) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: Some(stop_price),
            order_side: OrderSide::Sell,
            order_type: OrderType::StopLossLimit,
            time_in_force,
            new_client_order_id: None,
        };
        let order = build_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    /// Place a custom order
    ///
    /// # Errors
    ///
    /// Returns an error if the custom order cannot be placed.
    #[allow(clippy::too_many_arguments)]
    pub async fn custom_order<S, F>(
        &self,
        symbol: S,
        qty: F,
        price: f64,
        stop_price: Option<f64>,
        order_side: OrderSide,
        order_type: OrderType,
        time_in_force: TimeInForce,
        new_client_order_id: Option<String>,
    ) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price,
            order_side,
            order_type,
            time_in_force,
            new_client_order_id,
        };
        let order = build_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test custom order
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the test custom order cannot be placed.
    #[allow(clippy::too_many_arguments)]
    pub async fn test_custom_order<S, F>(
        &self,
        symbol: S,
        qty: F,
        price: f64,
        stop_price: Option<f64>,
        order_side: OrderSide,
        order_type: OrderType,
        time_in_force: TimeInForce,
        new_client_order_id: Option<String>,
    ) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price,
            order_side,
            order_type,
            time_in_force,
            new_client_order_id,
        };
        let order = build_order(order);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    /// Check an order's status
    ///
    /// # Errors
    ///
    /// Returns an error if the order status cannot be checkeds
    pub async fn cancel_order<S>(&self, symbol: S, order_id: u64) -> Result<OrderCanceled>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("orderId".into(), order_id.to_string());

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .delete_signed(API::Spot(Spot::Order), Some(request))
            .await
    }

    /// Cancel an order based on the original client order id
    ///
    /// # Errors
    ///
    /// Returns an error if the order cannot be cancelled
    pub async fn cancel_order_with_client_id<S>(
        &self,
        symbol: S,
        orig_client_order_id: String,
    ) -> Result<OrderCanceled>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("origClientOrderId".into(), orig_client_order_id);

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .delete_signed(API::Spot(Spot::Order), Some(request))
            .await
    }

    /// Place a test cancel order
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the test cancel order cannot be placed.
    pub async fn test_cancel_order<S>(&self, symbol: S, order_id: u64) -> Result<()>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());
        parameters.insert("orderId".into(), order_id.to_string());
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .delete_signed::<Empty>(API::Spot(Spot::OrderTest), Some(request))
            .await
            .map(|_| ())
    }

    /// Trade history
    ///
    /// # Errors
    ///
    /// Returns an error if the trade history cannot be retrieved
    pub async fn trade_history<S>(&self, symbol: S) -> Result<Vec<TradeHistory>>
    where
        S: Into<String>,
    {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert("symbol".into(), symbol.into());

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Spot(Spot::MyTrades), Some(request))
            .await
    }
}

fn build_quote_quantity_order(order: OrderQuoteQuantityRequest) -> BTreeMap<String, String> {
    let mut order_parameters: BTreeMap<String, String> = BTreeMap::new();

    order_parameters.insert("symbol".into(), order.symbol);
    order_parameters.insert("side".into(), order.order_side.to_string());
    order_parameters.insert("type".into(), order.order_type.to_string());
    order_parameters.insert("quoteOrderQty".into(), order.quote_order_qty.to_string());

    if order.price != 0.0 {
        order_parameters.insert("price".into(), order.price.to_string());
        order_parameters.insert("timeInForce".into(), order.time_in_force.to_string());
    }

    if let Some(client_order_id) = order.new_client_order_id {
        order_parameters.insert("newClientOrderId".into(), client_order_id);
    }

    order_parameters
}

fn build_order(order: OrderRequest) -> BTreeMap<String, String> {
    let mut order_parameters: BTreeMap<String, String> = BTreeMap::new();

    order_parameters.insert("symbol".into(), order.symbol);
    order_parameters.insert("side".into(), order.order_side.to_string());
    order_parameters.insert("type".into(), order.order_type.to_string());
    order_parameters.insert("quantity".into(), order.qty.to_string());

    if let Some(stop_price) = order.stop_price {
        order_parameters.insert("stopPrice".into(), stop_price.to_string());
    }

    if order.price != 0.0 {
        order_parameters.insert("price".into(), order.price.to_string());
        order_parameters.insert("timeInForce".into(), order.time_in_force.to_string());
    }

    if let Some(client_order_id) = order.new_client_order_id {
        order_parameters.insert("newClientOrderId".into(), client_order_id);
    }

    order_parameters
}
