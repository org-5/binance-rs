use std::cmp::min;
use std::collections::BTreeMap;
use std::fmt::Display;

use error_chain::bail;
use humantime::format_duration;
use tracing::debug;

use crate::api::Futures;
use crate::api::Spot;
use crate::api::API;
use crate::client::Client;
use crate::errors::Result;
use crate::model::AccountInformation;
use crate::model::Balance;
use crate::model::Empty;
use crate::model::HistoricalDataDownloadId;
use crate::model::Order;
use crate::model::OrderCanceled;
use crate::model::TradeHistory;
use crate::model::Transaction;
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
    // Account Information
    pub async fn get_account(&self) -> Result<AccountInformation> {
        let request = build_signed_request(BTreeMap::new(), self.recv_window)?;
        self.client
            .get_signed(API::Spot(Spot::Account), Some(request))
            .await
    }

    // Balance for a single Asset
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

    // Current open orders for ONE symbol
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

    // All current open orders
    pub async fn get_all_open_orders(&self) -> Result<Vec<Order>> {
        let parameters: BTreeMap<String, String> = BTreeMap::new();

        let request = build_signed_request(parameters, self.recv_window)?;
        self.client
            .get_signed(API::Spot(Spot::OpenOrders), Some(request))
            .await
    }

    // Cancel all open orders for a single symbol
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

    // Check an order's status
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

    // Place a LIMIT order - BUY
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
        let order = self.build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test limit order - BUY
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
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
        let order = self.build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    // Place a LIMIT order - SELL
    pub async fn limit_sell<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: None,
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test LIMIT order - SELL
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    pub async fn test_limit_sell<S, F>(&self, symbol: S, qty: F, price: f64) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: None,
            order_side: OrderSide::Sell,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    // Place a MARKET order - BUY
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
        let order = self.build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test MARKET order - BUY
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
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
        let order = self.build_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    // Place a MARKET order with quote quantity - BUY
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
        let order = self.build_quote_quantity_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test MARKET order with quote quantity - BUY
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
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
        let order = self.build_quote_quantity_order(buy);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    // Place a MARKET order - SELL
    pub async fn market_sell<S, F>(&self, symbol: S, qty: F) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price: 0.0,
            stop_price: None,
            order_side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test MARKET order - SELL
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    pub async fn test_market_sell<S, F>(&self, symbol: S, qty: F) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price: 0.0,
            stop_price: None,
            order_side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    // Place a MARKET order with quote quantity - SELL
    pub async fn market_sell_using_quote_quantity<S, F>(
        &self,
        symbol: S,
        quote_order_qty: F,
    ) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell = OrderQuoteQuantityRequest {
            symbol: symbol.into(),
            quote_order_qty: quote_order_qty.into(),
            price: 0.0,
            order_side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = self.build_quote_quantity_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test MARKET order with quote quantity - SELL
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
    pub async fn test_market_sell_using_quote_quantity<S, F>(
        &self,
        symbol: S,
        quote_order_qty: F,
    ) -> Result<()>
    where
        S: Into<String>,
        F: Into<f64>,
    {
        let sell = OrderQuoteQuantityRequest {
            symbol: symbol.into(),
            quote_order_qty: quote_order_qty.into(),
            price: 0.0,
            order_side: OrderSide::Sell,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::GTC,
            new_client_order_id: None,
        };
        let order = self.build_quote_quantity_order(sell);
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
    /// use binance::account::*;
    ///
    /// fn main() {
    ///     let api_key = Some("api_key".into());
    ///     let secret_key = Some("secret_key".into());
    ///     let account: Account = Binance::new(api_key, secret_key).unwrap();
    ///     let result = account.stop_limit_buy_order("LTCBTC", 1, 0.1, 0.09, TimeInForce::GTC);
    /// }
    /// ```
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
        let sell = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: Some(stop_price),
            order_side: OrderSide::Buy,
            order_type: OrderType::StopLossLimit,
            time_in_force,
            new_client_order_id: None,
        };
        let order = self.build_order(sell);
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
    /// use binance::account::*;
    ///
    /// fn main() {
    ///     let api_key = Some("api_key".into());
    ///     let secret_key = Some("secret_key".into());
    ///     let account: Account = Binance::new(api_key, secret_key).unwrap();
    ///     let result = account.test_stop_limit_buy_order("LTCBTC", 1, 0.1, 0.09, TimeInForce::GTC);
    /// }
    /// ```
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
        let sell = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: Some(stop_price),
            order_side: OrderSide::Buy,
            order_type: OrderType::StopLossLimit,
            time_in_force,
            new_client_order_id: None,
        };
        let order = self.build_order(sell);
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
    /// use binance::account::*;
    ///
    /// fn main() {
    ///     let api_key = Some("api_key".into());
    ///     let secret_key = Some("secret_key".into());
    ///     let account: Account = Binance::new(api_key, secret_key).unwrap();
    ///     let result = account.stop_limit_sell_order("LTCBTC", 1, 0.1, 0.09, TimeInForce::GTC);
    /// }
    /// ```
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
        let sell = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: Some(stop_price),
            order_side: OrderSide::Sell,
            order_type: OrderType::StopLossLimit,
            time_in_force,
            new_client_order_id: None,
        };
        let order = self.build_order(sell);
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
    /// use binance::account::*;
    ///
    /// fn main() {
    ///     let api_key = Some("api_key".into());
    ///     let secret_key = Some("secret_key".into());
    ///     let account: Account = Binance::new(api_key, secret_key).unwrap();
    ///     let result = account.test_stop_limit_sell_order("LTCBTC", 1, 0.1, 0.09, TimeInForce::GTC);
    /// }
    /// ```
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
        let sell = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price: Some(stop_price),
            order_side: OrderSide::Sell,
            order_type: OrderType::StopLossLimit,
            time_in_force,
            new_client_order_id: None,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    /// Place a custom order
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
        let sell = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price,
            order_side,
            order_type,
            time_in_force,
            new_client_order_id,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed(API::Spot(Spot::Order), request)
            .await
    }

    /// Place a test custom order
    ///
    /// This order is sandboxed: it is validated, but not sent to the matching
    /// engine.
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
        let sell = OrderRequest {
            symbol: symbol.into(),
            qty: qty.into(),
            price,
            stop_price,
            order_side,
            order_type,
            time_in_force,
            new_client_order_id,
        };
        let order = self.build_order(sell);
        let request = build_signed_request(order, self.recv_window)?;
        self.client
            .post_signed::<Empty>(API::Spot(Spot::OrderTest), request)
            .await
            .map(|_| ())
    }

    // Check an order's status
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

    // Trade history
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

    fn build_order(&self, order: OrderRequest) -> BTreeMap<String, String> {
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

    fn build_quote_quantity_order(
        &self,
        order: OrderQuoteQuantityRequest,
    ) -> BTreeMap<String, String> {
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

    pub async fn download_hist_data_get_download_id(
        &self,
        symbol: &str,
        start_time: u128,
        end_time: u128,
        data_type: &str,
        timestamp: u128,
    ) -> Result<Vec<HistoricalDataDownloadId>> {
        let duration = std::time::Duration::from_millis((end_time - start_time) as u64);
        debug!(
            "Downloading historical data for {} from {} to {}, duration: {}",
            symbol,
            start_time,
            end_time,
            format_duration(duration)
        );

        let mut ids: Vec<HistoricalDataDownloadId> = Vec::new();
        // Split in 3 months chunks
        // 3 months in milliseconds
        let three_months_duration: u128 = 3 * 30 * 24 * 60 * 60 * 1000;
        let mut tot_duration = end_time - start_time;
        let mut start_time = start_time;
        let mut this_duration = min(tot_duration, three_months_duration);

        while tot_duration > 0 {
            let mut parameters: BTreeMap<String, String> = BTreeMap::new();
            parameters.insert("symbol".into(), symbol.into());
            parameters.insert("startTime".into(), start_time.to_string());
            parameters.insert("endTime".into(), (start_time + this_duration).to_string());
            parameters.insert("dataType".into(), data_type.into());
            parameters.insert("timestamp".into(), timestamp.to_string());

            let request = build_signed_request(parameters, self.recv_window)?;

            let res: HistoricalDataDownloadId = self
                .client
                .post_signed(API::Futures(Futures::HistoricalDataDownloadId), request)
                .await?;

            ids.push(res);

            start_time += this_duration;
            tot_duration -= this_duration;
            this_duration = min(tot_duration, three_months_duration);
        }

        Ok(ids)
    }

    // pub fn download_hist_data_get_download_link(
    //     &self,
    //     download_id: &str,
    //     timestamp: u128,
    // ) -> Result<String> {
    //     let mut parameters: BTreeMap<String, String> = BTreeMap::new();
    //     parameters.insert("downloadId".into(), download_id.into());
    //     parameters.insert("timestamp".into(), timestamp.to_string());
    //     let start_time = Instant::now();

    //     let res = loop {
    //         let request = build_signed_request(parameters.clone(),
    // self.recv_window)?;

    //         let res: HistoricalDataDownloadLink = self.client.get_signed(
    //             API::Futures(Futures::HistoricalDataDownloadLink),
    //             Some(request),
    //         )?;

    //         // result is Link is preparing, please try again later
    //         if res
    //             .link
    //             .contains("Link is preparing; please request later.")
    //         {
    //             info!(
    //                 res.link,
    //                 "Link is preparing; please request later, waited for a total
    // of {:?} so far. sleeping 60s",                 Instant::now() -
    // start_time             );
    //             thread::sleep(Duration::from_secs(60));
    //             continue;
    //         }
    //         if !res.link.starts_with("https://") {
    //             // "received something, but not a link".into() show link
    //             return Err(format!("received something, but not a link: {}",
    // res.link).into());         }

    //         break res.link;
    //     };

    //     Ok(res)
    // }

    // pub async fn download_hist_data_file(&self, url: &str, path: PathBuf) ->
    // Result<PathBuf> {     if path.ends_with("/") {
    //         return Err("must be a path to a file".into());
    //     }
    //     if path.extension().is_none() {
    //         return Err("must have a .tar.gz extension".into());
    //     }

    //     let resp = reqwest::get(url).await?;

    //     let len: usize = resp
    //         .headers()
    //         .get("Content-Length")
    //         .unwrap()
    //         .to_str()
    //         .unwrap()
    //         .parse()
    //         .unwrap();

    //     let mut buffer = [0; 10_000];
    //     let mut reader = resp.into_reader();
    //     let mut cursor = 0;

    //     fs::create_dir_all(path.parent().unwrap())?;

    //     let mut file = fs::File::create(path.clone()).unwrap();

    //     loop {
    //         let b_len = reader.read(&mut buffer).unwrap();

    //         // write to file
    //         file.write_all(&buffer[0..b_len]).unwrap();

    //         cursor += b_len;
    //         if cursor >= len {
    //             break;
    //         }
    //     }

    //     Ok(path)
    // }
}
