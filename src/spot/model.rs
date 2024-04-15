use std::convert::TryFrom;

use serde::Deserialize;
use serde::Serialize;
use serde_json::from_value;
use serde_json::Value;

use crate::errors::Error;
use crate::errors::ErrorKind;
use crate::errors::Result;
use crate::model::string_or_float;
pub use crate::model::Asks;
pub use crate::model::Bids;
pub use crate::model::BookTickers;
pub use crate::model::Filters;
pub use crate::model::KlineSummaries;
pub use crate::model::KlineSummary;
pub use crate::model::RateLimit;
pub use crate::model::ServerTime;
pub use crate::model::SymbolInfo;
pub use crate::model::SymbolPrice;
pub use crate::model::Tickers;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInformation {
    pub timezone: String,
    pub server_time: u64,
    pub rate_limits: Vec<RateLimit>,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub symbol: String,
    pub status: String,
    pub base_asset: String,
    pub base_asset_precision: u64,
    pub quote_asset: String,
    pub quote_precision: u64,
    pub order_types: Vec<String>,
    pub iceberg_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub filters: Vec<Filters>,
}

impl SymbolInfo for Symbol {
    fn ticker(&self) -> &str {
        &self.symbol
    }

    fn status(&self) -> &str {
        &self.status
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountInformation {
    pub maker_commission: f32,
    pub taker_commission: f32,
    pub buyer_commission: f32,
    pub seller_commission: f32,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub balances: Vec<Balance>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub symbol: String,
    pub order_id: u64,
    pub order_list_id: i64,
    pub client_order_id: String,
    #[serde(with = "string_or_float")]
    pub price: f64,
    pub orig_qty: String,
    pub executed_qty: String,
    pub cummulative_quote_qty: String,
    pub status: String,
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub side: String,
    #[serde(with = "string_or_float")]
    pub stop_price: f64,
    pub iceberg_qty: String,
    pub time: u64,
    pub update_time: u64,
    pub is_working: bool,
    pub orig_quote_order_qty: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderCanceled {
    pub symbol: String,
    pub orig_client_order_id: Option<String>,
    pub order_id: Option<u64>,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub symbol: String,
    pub order_id: u64,
    pub order_list_id: Option<i64>,
    pub client_order_id: String,
    pub transact_time: u64,
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    #[serde(with = "string_or_float")]
    pub cummulative_quote_qty: f64,
    #[serde(with = "string_or_float", default = "default_stop_price")]
    pub stop_price: f64,
    pub status: String,
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub side: String,
    pub fills: Option<Vec<FillInfo>>,
}

fn default_stop_price() -> f64 {
    0.0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FillInfo {
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    pub qty: f64,
    #[serde(with = "string_or_float")]
    pub commission: f64,
    pub commission_asset: String,
    pub trade_id: Option<u64>,
}
/// Response to a test order (endpoint /api/v3/order/test).
///
/// Currently, the API responds {} on a successfull test transaction,
/// hence this struct has no fields.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestResponse {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    pub last_update_id: u64,
    pub bids: Vec<Bids>,
    pub asks: Vec<Asks>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserDataStream {
    pub listen_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Success {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum Prices {
    AllPrices(Vec<SymbolPrice>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AveragePrice {
    pub mins: u64,
    #[serde(with = "string_or_float")]
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradeHistory {
    pub id: u64,
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    pub qty: f64,
    pub commission: String,
    pub commission_asset: String,
    pub time: u64,
    pub is_buyer: bool,
    pub is_maker: bool,
    pub is_best_match: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceStats {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub weighted_avg_price: String,
    #[serde(with = "string_or_float")]
    pub prev_close_price: f64,
    #[serde(with = "string_or_float")]
    pub last_price: f64,
    #[serde(with = "string_or_float")]
    pub bid_price: f64,
    #[serde(with = "string_or_float")]
    pub ask_price: f64,
    #[serde(with = "string_or_float")]
    pub open_price: f64,
    #[serde(with = "string_or_float")]
    pub high_price: f64,
    #[serde(with = "string_or_float")]
    pub low_price: f64,
    #[serde(with = "string_or_float")]
    pub volume: f64,
    pub open_time: u64,
    pub close_time: u64,
    pub first_id: i64,
    pub last_id: i64,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AggTrade {
    #[serde(rename = "T")]
    pub time: u64,
    #[serde(rename = "a")]
    pub agg_id: u64,
    #[serde(rename = "f")]
    pub first_id: u64,
    #[serde(rename = "l")]
    pub last_id: u64,
    #[serde(rename = "m")]
    pub maker: bool,
    #[serde(rename = "M")]
    pub best_match: bool,
    #[serde(rename = "p", with = "string_or_float")]
    pub price: f64,
    #[serde(rename = "q", with = "string_or_float")]
    pub qty: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderTradeEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "c")]
    pub new_client_order_id: String,

    #[serde(rename = "S")]
    pub side: String,

    #[serde(rename = "o")]
    pub order_type: String,

    #[serde(rename = "f")]
    pub time_in_force: String,

    #[serde(rename = "q")]
    pub qty: String,

    #[serde(rename = "p")]
    pub price: String,

    #[serde(skip, rename = "P")]
    pub p_ignore: String,

    #[serde(skip, rename = "F")]
    pub f_ignore: String,

    #[serde(skip)]
    pub g: i32,

    #[serde(skip, rename = "C")]
    pub c_ignore: Option<String>,

    #[serde(rename = "x")]
    pub execution_type: String,

    #[serde(rename = "X")]
    pub order_status: String,

    #[serde(rename = "r")]
    pub order_reject_reason: String,

    #[serde(rename = "i")]
    pub order_id: u64,

    #[serde(rename = "l")]
    pub qty_last_filled_trade: String,

    #[serde(rename = "z")]
    pub accumulated_qty_filled_trades: String,

    #[serde(rename = "L")]
    pub price_last_filled_trade: String,

    #[serde(rename = "n")]
    pub commission: String,

    #[serde(skip, rename = "N")]
    pub asset_commisioned: Option<String>,

    #[serde(rename = "T")]
    pub trade_order_time: u64,

    #[serde(rename = "t")]
    pub trade_id: i64,

    #[serde(skip, rename = "I")]
    pub i_ignore: u64,

    #[serde(skip)]
    pub w: bool,

    #[serde(rename = "m")]
    pub is_buyer_maker: bool,

    #[serde(skip, rename = "M")]
    pub m_ignore: bool,
}

fn get_value(row: &[Value], index: usize, name: &'static str) -> Result<Value> {
    Ok(row
        .get(index)
        .ok_or_else(|| ErrorKind::KlineValueMissingError(index, name))?
        .clone())
}

impl TryFrom<&Vec<Value>> for KlineSummary {
    type Error = Error;

    fn try_from(row: &Vec<Value>) -> Result<Self> {
        Ok(Self {
            open_time: from_value(get_value(row, 0, "open_time")?)?,
            open: from_value(get_value(row, 1, "open")?)?,
            high: from_value(get_value(row, 2, "high")?)?,
            low: from_value(get_value(row, 3, "low")?)?,
            close: from_value(get_value(row, 4, "close")?)?,
            volume: from_value(get_value(row, 5, "volume")?)?,
            close_time: from_value(get_value(row, 6, "close_time")?)?,
            quote_asset_volume: from_value(get_value(row, 7, "quote_asset_volume")?)?,
            number_of_trades: from_value(get_value(row, 8, "number_of_trades")?)?,
            taker_buy_base_asset_volume: from_value(get_value(
                row,
                9,
                "taker_buy_base_asset_volume",
            )?)?,
            taker_buy_quote_asset_volume: from_value(get_value(
                row,
                10,
                "taker_buy_quote_asset_volume",
            )?)?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoricalDataDownloadId {
    pub id: u128,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoricalDataDownloadLink {
    pub link: String,
}
