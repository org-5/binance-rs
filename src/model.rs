use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Clone)]
pub struct Empty {}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    pub server_time: u64,
}

pub trait SymbolInfo {
    fn ticker(&self) -> &str;
    fn status(&self) -> &str;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SymbolPrice {
    pub symbol: String,
    #[serde(with = "string_or_float")]
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "filterType")]
pub enum Filters {
    #[serde(rename = "PRICE_FILTER")]
    #[serde(rename_all = "camelCase")]
    PriceFilter {
        min_price: String,
        max_price: String,
        tick_size: String,
    },
    #[serde(rename = "PERCENT_PRICE")]
    #[serde(rename_all = "camelCase")]
    PercentPrice {
        multiplier_up: String,
        multiplier_down: String,
        avg_price_mins: Option<f64>,
    },
    #[serde(rename = "PERCENT_PRICE_BY_SIDE")]
    #[serde(rename_all = "camelCase")]
    PercentPriceBySide {
        bid_multiplier_up: String,
        bid_multiplier_down: String,
        ask_multiplier_up: String,
        ask_multiplier_down: String,
        avg_price_mins: Option<f64>,
    },
    #[serde(rename = "LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    LotSize {
        min_qty: String,
        max_qty: String,
        step_size: String,
    },
    #[serde(rename = "MIN_NOTIONAL")]
    #[serde(rename_all = "camelCase")]
    MinNotional {
        notional: Option<String>,
        min_notional: Option<String>,
        apply_to_market: Option<bool>,
        avg_price_mins: Option<f64>,
    },
    #[serde(rename = "NOTIONAL")]
    #[serde(rename_all = "camelCase")]
    Notional {
        notional: Option<String>,
        min_notional: Option<String>,
        apply_to_market: Option<bool>,
        avg_price_mins: Option<f64>,
    },
    #[serde(rename = "ICEBERG_PARTS")]
    #[serde(rename_all = "camelCase")]
    IcebergParts { limit: Option<u16> },
    #[serde(rename = "MAX_NUM_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumOrders { max_num_orders: Option<u16> },
    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumAlgoOrders { max_num_algo_orders: Option<u16> },
    #[serde(rename = "MAX_NUM_ICEBERG_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumIcebergOrders { max_num_iceberg_orders: u16 },
    #[serde(rename = "MAX_POSITION")]
    #[serde(rename_all = "camelCase")]
    MaxPosition { max_position: String },
    #[serde(rename = "MARKET_LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    MarketLotSize {
        min_qty: String,
        max_qty: String,
        step_size: String,
    },
    #[serde(rename = "TRAILING_DELTA")]
    #[serde(rename_all = "camelCase")]
    TrailingData {
        min_trailing_above_delta: Option<u16>,
        max_trailing_above_delta: Option<u16>,
        min_trailing_below_delta: Option<u16>,
        max_trailing_below_delta: Option<u16>,
    },
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRates {
    pub symbol: String,
    pub standard_commission: Commission,
    pub tax_commission: Commission,
    pub discount: Discount,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commission {
    pub maker: String,
    pub taker: String,
    pub buyer: String,
    pub seller: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Discount {
    pub enabled_for_account: bool,
    pub enabled_for_symbol: bool,
    pub discount_asset: String,
    pub discount: String,
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
pub enum SpotFuturesTransferType {
    SpotToUsdtFutures = 1,
    UsdtFuturesToSpot = 2,
    SpotToCoinFutures = 3,
    CoinFuturesToSpot = 4,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionId {
    pub tran_id: u64,
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
pub struct RateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub interval_num: u16,
    pub limit: u64,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Bids {
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub qty: Decimal,
}

impl Bids {
    #[must_use]
    pub fn new(price: Decimal, qty: Decimal) -> Bids {
        Bids { price, qty }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum BookTickers {
    AllBookTickers(Vec<Tickers>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum KlineSummaries {
    AllKlineSummaries(Vec<KlineSummary>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tickers {
    pub symbol: String,
    #[serde(with = "string_or_float")]
    pub bid_price: f64,
    #[serde(with = "string_or_float")]
    pub bid_qty: f64,
    #[serde(with = "string_or_float")]
    pub ask_price: f64,
    #[serde(with = "string_or_float")]
    pub ask_qty: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asks {
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub qty: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineSummary {
    pub open_time: i64,

    pub open: String,

    pub high: String,

    pub low: String,

    pub close: String,

    pub volume: String,

    pub close_time: i64,

    pub quote_asset_volume: String,

    pub number_of_trades: i64,

    pub taker_buy_base_asset_volume: String,

    pub taker_buy_quote_asset_volume: String,
}

/// Part of the Savings API get all coins response
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct Network {
    pub address_regex: String,
    pub coin: String,
    /// shown only when "depositEnable" is false.
    pub deposit_desc: Option<String>,
    pub deposit_enable: bool,
    pub is_default: bool,
    pub memo_regex: String,
    /// min number for balance confirmation
    pub min_confirm: u32,
    pub name: String,
    pub network: String,
    pub reset_address_status: bool,
    pub special_tips: Option<String>,
    /// confirmation number for balance unlock
    pub un_lock_confirm: u32,
    /// shown only when "withdrawEnable" is false.
    pub withdraw_desc: Option<String>,
    pub withdraw_enable: bool,
    #[serde(with = "string_or_float")]
    pub withdraw_fee: f64,
    #[serde(with = "string_or_float")]
    pub withdraw_min: f64,
    // pub insert_time: Option<u64>, //commented out for now, because they are not inside the
    // actual response (only the api doc example) pub update_time: Option<u64>,
    pub withdraw_integer_multiple: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetDetail {
    #[serde(with = "string_or_float")]
    pub min_withdraw_amount: f64,
    /// false if ALL of networks' are false
    pub deposit_status: bool,
    #[serde(with = "string_or_float")]
    pub withdraw_fee: f64,
    /// false if ALL of networks' are false
    pub withdraw_status: bool,
    /// reason
    pub deposit_tip: Option<String>,
}

/// Response to the Savings API get all coins request
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct CoinInfo {
    pub coin: String,
    pub deposit_all_enable: bool,
    #[serde(with = "string_or_float")]
    pub free: f64,
    #[serde(with = "string_or_float")]
    pub freeze: f64,
    #[serde(with = "string_or_float")]
    pub ipoable: f64,
    #[serde(with = "string_or_float")]
    pub ipoing: f64,
    pub is_legal_money: bool,
    #[serde(with = "string_or_float")]
    pub locked: f64,
    pub name: String,
    pub network_list: Vec<Network>,
    #[serde(with = "string_or_float")]
    pub storage: f64,
    pub trading: bool,
    pub withdraw_all_enable: bool,
    #[serde(with = "string_or_float")]
    pub withdrawing: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddress {
    pub address: String,
    pub coin: String,
    pub tag: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdateEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "a")]
    pub data: AccountUpdateDataEvent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdateDataEvent {
    #[serde(rename = "m")]
    pub reason: String,

    #[serde(rename = "B")]
    pub balances: Vec<EventBalance>,

    #[serde(rename = "P")]
    pub positions: Vec<EventPosition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventBalance {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "wb")]
    pub wallet_balance: String,
    #[serde(rename = "cw")]
    pub cross_wallet_balance: String,
    #[serde(rename = "bc")]
    pub balance_change: String, // Balance Change except PnL and Commission
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventPosition {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "pa")]
    pub position_amount: String,
    #[serde(rename = "ep")]
    pub entry_price: String,
    #[serde(rename = "cr")]
    pub accumulated_realized: String, // (Pre-fee) Accumulated Realized
    #[serde(rename = "up")]
    pub unrealized_pnl: String,
    #[serde(rename = "mt")]
    pub margin_type: String,
    #[serde(rename = "iw")]
    pub isolated_wallet: String,
    #[serde(rename = "ps")]
    pub position_side: String,
}

/// The Aggregate Trade Streams push trade information that is aggregated for a
/// single taker order.
///
/// Stream Name: \<symbol\>@aggTrade
///
/// Update Speed: Real-time
///
/// <https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#aggregate-trade-streams>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AggrTradesEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "a")]
    pub aggregated_trade_id: u64,

    #[serde(rename = "p")]
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,

    #[serde(rename = "q")]
    #[serde(with = "rust_decimal::serde::str")]
    pub qty: Decimal,

    #[serde(rename = "f")]
    pub first_break_trade_id: u64,

    #[serde(rename = "l")]
    pub last_break_trade_id: u64,

    #[serde(rename = "T")]
    pub trade_order_time: u64,

    #[serde(rename = "m")]
    pub is_buyer_maker: bool,

    #[serde(skip, rename = "M")]
    pub m_ignore: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceUpdateEvent {
    #[serde(rename = "B")]
    pub balance: Vec<EventBalance>,

    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "u")]
    pub last_account_update_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookTickerEvent {
    #[serde(rename = "u")]
    pub update_id: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "b")]
    pub best_bid: String,

    #[serde(rename = "B")]
    pub best_bid_qty: String,

    #[serde(rename = "a")]
    pub best_ask: String,

    #[serde(rename = "A")]
    pub best_ask_qty: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DayTickerEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "p")]
    pub price_change: String,

    #[serde(rename = "P")]
    pub price_change_percent: String,

    #[serde(rename = "w")]
    pub average_price: String,

    #[serde(rename = "x")]
    pub prev_close: String,

    #[serde(rename = "c")]
    pub current_close: String,

    #[serde(rename = "Q")]
    pub current_close_qty: String,

    #[serde(rename = "b")]
    pub best_bid: String,

    #[serde(rename = "B")]
    pub best_bid_qty: String,

    #[serde(rename = "a")]
    pub best_ask: String,

    #[serde(rename = "A")]
    pub best_ask_qty: String,

    #[serde(rename = "o")]
    pub open: String,

    #[serde(rename = "h")]
    pub high: String,

    #[serde(rename = "l")]
    pub low: String,

    #[serde(rename = "v")]
    pub volume: String,

    #[serde(rename = "q")]
    pub quote_volume: String,

    #[serde(rename = "O")]
    pub open_time: u64,

    #[serde(rename = "C")]
    pub close_time: u64,

    #[serde(rename = "F")]
    pub first_trade_id: i64,

    #[serde(rename = "L")]
    pub last_trade_id: i64,

    #[serde(rename = "n")]
    pub num_trades: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepthOrderBookEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "U")]
    pub first_update_id: u64,

    #[serde(rename = "u")]
    pub final_update_id: u64,

    #[serde(rename = "pu")]
    #[serde(default)]
    pub previous_final_update_id: Option<u64>,

    #[serde(rename = "b")]
    pub bids: Vec<Bids>,

    #[serde(rename = "a")]
    pub asks: Vec<Asks>,
}

// https://binance-docs.github.io/apidocs/delivery/en/#index-kline-candlestick-streams
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexKline {
    #[serde(rename = "t")]
    pub start_time: i64,

    #[serde(rename = "T")]
    pub end_time: i64,

    #[serde(skip, rename = "s")]
    pub ignore_me: String,

    #[serde(rename = "i")]
    pub interval: String,

    #[serde(rename = "f")]
    pub first_trade_id: i64,

    #[serde(rename = "L")]
    pub last_trade_id: i64,

    #[serde(rename = "o")]
    pub open: String,

    #[serde(rename = "c")]
    pub close: String,

    #[serde(rename = "h")]
    pub high: String,

    #[serde(rename = "l")]
    pub low: String,

    #[serde(rename = "v")]
    pub volume: String,

    #[serde(rename = "n")]
    pub number_of_trades: i64,

    #[serde(rename = "x")]
    pub is_final_bar: bool,

    #[serde(skip, rename = "q")]
    pub ignore_me2: String,

    #[serde(skip, rename = "V")]
    pub ignore_me3: String,

    #[serde(skip, rename = "Q")]
    pub ignore_me4: String,

    #[serde(skip, rename = "B")]
    pub ignore_me5: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Kline {
    #[serde(rename = "t")]
    pub open_time: i64,

    #[serde(rename = "T")]
    pub close_time: i64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "i")]
    pub interval: String,

    #[serde(rename = "f")]
    pub first_trade_id: i64,

    #[serde(rename = "L")]
    pub last_trade_id: i64,

    #[serde(rename = "o")]
    pub open: String,

    #[serde(rename = "c")]
    pub close: String,

    #[serde(rename = "h")]
    pub high: String,

    #[serde(rename = "l")]
    pub low: String,

    #[serde(rename = "v")]
    pub volume: String,

    #[serde(rename = "n")]
    pub number_of_trades: i64,

    #[serde(rename = "x")]
    pub is_final_bar: bool,

    #[serde(rename = "q")]
    pub quote_asset_volume: String,

    #[serde(rename = "V")]
    pub taker_buy_base_asset_volume: String,

    #[serde(rename = "Q")]
    pub taker_buy_quote_asset_volume: String,

    #[serde(skip, rename = "B")]
    pub ignore_me: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KlineEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "k")]
    pub kline: Kline,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousKline {
    #[serde(rename = "t")]
    pub start_time: i64,

    #[serde(rename = "T")]
    pub end_time: i64,

    #[serde(rename = "i")]
    pub interval: String,

    #[serde(rename = "f")]
    pub first_trade_id: i64,

    #[serde(rename = "L")]
    pub last_trade_id: i64,

    #[serde(rename = "o")]
    pub open: String,

    #[serde(rename = "c")]
    pub close: String,

    #[serde(rename = "h")]
    pub high: String,

    #[serde(rename = "l")]
    pub low: String,

    #[serde(rename = "v")]
    pub volume: String,

    #[serde(rename = "n")]
    pub number_of_trades: i64,

    #[serde(rename = "x")]
    pub is_final_bar: bool,

    #[serde(rename = "q")]
    pub quote_volume: String,

    #[serde(rename = "V")]
    pub active_buy_volume: String,

    #[serde(rename = "Q")]
    pub active_volume_buy_quote: String,

    #[serde(skip, rename = "B")]
    pub ignore_me: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousKlineEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "ps")]
    pub pair: String,

    #[serde(rename = "ct")]
    pub contract_type: String,

    #[serde(rename = "k")]
    pub kline: ContinuousKline,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexKlineEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "ps")]
    pub pair: String,

    #[serde(rename = "k")]
    pub kline: IndexKline,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiquidationOrder {
    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "S")]
    pub side: String,

    #[serde(rename = "o")]
    pub order_type: String,

    #[serde(rename = "f")]
    pub time_in_force: String,

    #[serde(rename = "q")]
    pub original_quantity: String,

    #[serde(rename = "p")]
    pub price: String,

    #[serde(rename = "ap")]
    pub average_price: String,

    #[serde(rename = "X")]
    pub order_status: String,

    #[serde(rename = "l")]
    pub order_last_filled_quantity: String,

    #[serde(rename = "z")]
    pub order_filled_accumulated_quantity: String,

    #[serde(rename = "T")]
    pub order_trade_time: u64,
}

// Object({"E": Number(1626118018407), "e": String("forceOrder"), "o":
// Object({"S": String("SELL"), "T": Number(1626118018404), "X":
// String("FILLED"), "ap": String("33028.07"), "f": String("IOC"), "l":
// String("0.010"), "o": String("LIMIT"), "p": String("32896.00"), "q":
// String("0.010"), "s": String("BTCUSDT"), "z": String("0.010")})}) https://binance-docs.github.io/apidocs/futures/en/#liquidation-order-streams

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiquidationEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "o")]
    pub liquidation_order: LiquidationOrder,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexPriceEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "i")]
    pub pair: String,

    #[serde(rename = "p")]
    pub price: String,
}

// https://binance-docs.github.io/apidocs/futures/en/#mark-price-stream
// https://binance-docs.github.io/apidocs/delivery/en/#mark-price-stream
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarkPriceEvent {
    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "P")]
    pub estimate_settle_price: String,

    #[serde(rename = "T")]
    pub next_funding_time: u64,

    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "i")]
    pub index_price: Option<String>,

    #[serde(rename = "p")]
    pub mark_price: String,

    #[serde(rename = "r")]
    pub funding_rate: String,

    #[serde(rename = "s")]
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MiniTickerEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "c")]
    pub close: String,

    #[serde(rename = "o")]
    pub open: String,

    #[serde(rename = "h")]
    pub high: String,

    #[serde(rename = "l")]
    pub low: String,

    #[serde(rename = "v")]
    pub volume: String,

    #[serde(rename = "q")]
    pub quote_volume: String,
}

/// The Trade Streams push raw trade information; each trade has a unique buyer
/// and seller.
///
/// Stream Name: \<symbol\>@trade
///
/// Update Speed: Real-time
///
/// <https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#trade-streams>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradeEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "t")]
    pub trade_id: u64,

    #[serde(rename = "p")]
    pub price: String,

    #[serde(rename = "q")]
    pub qty: String,

    #[serde(rename = "b")]
    pub buyer_order_id: u64,

    #[serde(rename = "a")]
    pub seller_order_id: u64,

    #[serde(rename = "T")]
    pub trade_order_time: u64,

    #[serde(rename = "m")]
    pub is_buyer_maker: bool,

    #[serde(skip, rename = "M")]
    pub m_ignore: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserDataStreamExpiredEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,
}

pub(crate) mod string_or_float {
    use std::fmt;

    use serde::de;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String(String),
            Float(f64),
        }

        match StringOrFloat::deserialize(deserializer)? {
            StringOrFloat::String(s) => {
                if s == "INF" {
                    Ok(f64::INFINITY)
                } else {
                    s.parse().map_err(de::Error::custom)
                }
            }
            StringOrFloat::Float(i) => Ok(i),
        }
    }
}

pub(crate) mod string_or_float_opt {
    use std::fmt;

    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        match value {
            Some(v) => crate::model::string_or_float::serialize(v, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String,
            Float,
        }

        Ok(Some(crate::model::string_or_float::deserialize(
            deserializer,
        )?))
    }
}

pub(crate) mod string_or_bool {
    use std::fmt;

    use serde::de;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String(String),
            Bool(bool),
        }

        match StringOrFloat::deserialize(deserializer)? {
            StringOrFloat::String(s) => s.parse().map_err(de::Error::custom),
            StringOrFloat::Bool(i) => Ok(i),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::model::AccountUpdateEvent;

    #[test]
    fn test_account_update_event() {
        let json = r#"
    {
  "e": "ACCOUNT_UPDATE",
  "E": 1564745798939,
  "T": 1564745798938,
  "a": {
    "m": "ORDER",
    "B": [
      {
        "a": "USDT",
        "wb": "122624.12345678",
        "cw": "100.12345678",
        "bc": "50.12345678"
      },
      {
        "a": "BUSD",
        "wb": "1.00000000",
        "cw": "0.00000000",
        "bc": "-49.12345678"
      }
    ],
    "P": [
      {
        "s": "BTCUSDT",
        "pa": "0",
        "ep": "0.00000",
        "cr": "200",
        "up": "0",
        "mt": "isolated",
        "iw": "0.00000000",
        "ps": "BOTH"
      },
      {
        "s": "BTCUSDT",
        "pa": "20",
        "ep": "6563.66500",
        "cr": "0",
        "up": "2850.21200",
        "mt": "isolated",
        "iw": "13200.70726908",
        "ps": "LONG"
      },
      {
        "s": "BTCUSDT",
        "pa": "-10",
        "ep": "6563.86000",
        "cr": "-45.04000000",
        "up": "-1423.15600",
        "mt": "isolated",
        "iw": "6570.42511771",
        "ps": "SHORT"
      }
    ]
  }
}
    "#;

        let res = r#"AccountUpdateEvent { event_type: "ACCOUNT_UPDATE", event_time: 1564745798939, data: AccountUpdateDataEvent { reason: "ORDER", balances: [EventBalance { asset: "USDT", wallet_balance: "122624.12345678", cross_wallet_balance: "100.12345678", balance_change: "50.12345678" }, EventBalance { asset: "BUSD", wallet_balance: "1.00000000", cross_wallet_balance: "0.00000000", balance_change: "-49.12345678" }], positions: [EventPosition { symbol: "BTCUSDT", position_amount: "0", entry_price: "0.00000", accumulated_realized: "200", unrealized_pnl: "0", margin_type: "isolated", isolated_wallet: "0.00000000", position_side: "BOTH" }, EventPosition { symbol: "BTCUSDT", position_amount: "20", entry_price: "6563.66500", accumulated_realized: "0", unrealized_pnl: "2850.21200", margin_type: "isolated", isolated_wallet: "13200.70726908", position_side: "LONG" }, EventPosition { symbol: "BTCUSDT", position_amount: "-10", entry_price: "6563.86000", accumulated_realized: "-45.04000000", unrealized_pnl: "-1423.15600", margin_type: "isolated", isolated_wallet: "6570.42511771", position_side: "SHORT" }] } }"#;
        let v: AccountUpdateEvent = serde_json::from_str(json).unwrap();
        assert_eq!(format!("{v:?}"), res);
        //let event =  from_value::<AccountUpdateEvent>(json).unwrap();
    }
}
