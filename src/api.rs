use crate::account::Account;
use crate::client::Client;
use crate::config::Config;
use crate::futures::account::FuturesAccount;
use crate::futures::general::FuturesGeneral;
use crate::futures::market::FuturesMarket;
use crate::futures::userstream::FuturesUserStream;
use crate::general::General;
use crate::market::Market;
use crate::userstream::UserStream;
use crate::savings::Savings;

#[allow(clippy::all)]
pub enum API {
    Spot(Spot),
    Savings(Sapi),
    Futures(Futures),
}

/// Endpoint for production and test orders.
///
/// Orders issued to test are validated, but not sent into the matching engine.
pub enum Spot {
    Ping,
    Time,
    ExchangeInfo,
    Depth,
    Trades,
    HistoricalTrades,
    AggTrades,
    Klines,
    AvgPrice,
    Ticker24hr,
    Price,
    BookTicker,
    Order,
    OrderTest,
    OpenOrders,
    AllOrders,
    Oco,
    OrderList,
    AllOrderList,
    OpenOrderList,
    Account,
    MyTrades,
    UserDataStream,
}

pub enum Sapi {
    AllCoins,
    AssetDetail,
    DepositAddress,
    SpotFuturesTransfer,
}

pub enum Futures {
    Ping,
    Time,
    ExchangeInfo,
    Depth,
    Trades,
    HistoricalTrades,
    AggTrades,
    Klines,
    ContinuousKlines,
    IndexPriceKlines,
    MarkPriceKlines,
    PremiumIndex,
    FundingRate,
    Ticker24hr,
    TickerPrice,
    BookTicker,
    AllForceOrders,
    AllOpenOrders,
    AllOrders,
    UserTrades,
    Order,
    PositionRisk,
    Balance,
    PositionSide,
    OpenInterest,
    OpenInterestHist,
    TopLongShortAccountRatio,
    TopLongShortPositionRatio,
    GlobalLongShortAccountRatio,
    TakerlongshortRatio,
    LvtKlines,
    IndexInfo,
    ChangeInitialLeverage,
    Account,
    OpenOrders,
    UserDataStream,
    Income,
    HistoricalDataDownloadId,
    HistoricalDataDownloadLink,
    DownloadLink(String),
}

impl From<API> for String {
    fn from(item: API) -> Self {
        match item {
            API::Spot(route) => match route {
                Spot::Ping => "/api/v3/ping".to_owned(),
                Spot::Time => "/api/v3/time".to_owned(),
                Spot::ExchangeInfo => "/api/v3/exchangeInfo".to_owned(),
                Spot::Depth => "/api/v3/depth".to_owned(),
                Spot::Trades => "/api/v3/trades".to_owned(),
                Spot::HistoricalTrades => "/api/v3/historicalTrades".to_owned(),
                Spot::AggTrades => "/api/v3/aggTrades".to_owned(),
                Spot::Klines => "/api/v3/klines".to_owned(),
                Spot::AvgPrice => "/api/v3/avgPrice".to_owned(),
                Spot::Ticker24hr => "/api/v3/ticker/24hr".to_owned(),
                Spot::Price => "/api/v3/ticker/price".to_owned(),
                Spot::BookTicker => "/api/v3/ticker/bookTicker".to_owned(),
                Spot::Order => "/api/v3/order".to_owned(),
                Spot::OrderTest => "/api/v3/order/test".to_owned(),
                Spot::OpenOrders => "/api/v3/openOrders".to_owned(),
                Spot::AllOrders => "/api/v3/allOrders".to_owned(),
                Spot::Oco => "/api/v3/order/oco".to_owned(),
                Spot::OrderList => "/api/v3/orderList".to_owned(),
                Spot::AllOrderList => "/api/v3/allOrderList".to_owned(),
                Spot::OpenOrderList => "/api/v3/openOrderList".to_owned(),
                Spot::Account => "/api/v3/account".to_owned(),
                Spot::MyTrades => "/api/v3/myTrades".to_owned(),
                Spot::UserDataStream => "/api/v3/userDataStream".to_owned(),
            },
            API::Savings(route) => match route {
                Sapi::AllCoins => "/sapi/v1/capital/config/getall".to_owned(),
                Sapi::AssetDetail => "/sapi/v1/asset/assetDetail".to_owned(),
                Sapi::DepositAddress => "/sapi/v1/capital/deposit/address".to_owned(),
                Sapi::SpotFuturesTransfer => "/sapi/v1/futures/transfer".to_owned(),
            },
            API::Futures(route) => match route {
                Futures::Ping => "/fapi/v1/ping".to_owned(),
                Futures::Time => "/fapi/v1/time".to_owned(),
                Futures::ExchangeInfo => "/fapi/v1/exchangeInfo".to_owned(),
                Futures::Depth => "/fapi/v1/depth".to_owned(),
                Futures::Trades => "/fapi/v1/trades".to_owned(),
                Futures::HistoricalTrades => "/fapi/v1/historicalTrades".to_owned(),
                Futures::AggTrades => "/fapi/v1/aggTrades".to_owned(),
                Futures::Klines => "/fapi/v1/klines".to_owned(),
                Futures::ContinuousKlines => "/fapi/v1/continuousKlines".to_owned(),
                Futures::IndexPriceKlines => "/fapi/v1/indexPriceKlines".to_owned(),
                Futures::MarkPriceKlines => "/fapi/v1/markPriceKlines".to_owned(),
                Futures::PremiumIndex => "/fapi/v1/premiumIndex".to_owned(),
                Futures::FundingRate => "/fapi/v1/fundingRate".to_owned(),
                Futures::Ticker24hr => "/fapi/v1/ticker/24hr".to_owned(),
                Futures::TickerPrice => "/fapi/v1/ticker/price".to_owned(),
                Futures::BookTicker => "/fapi/v1/ticker/bookTicker".to_owned(),
                Futures::AllForceOrders => "/fapi/v1/allForceOrders".to_owned(),
                Futures::AllOpenOrders => "/fapi/v1/allOpenOrders".to_owned(),
                Futures::AllOrders => "/fapi/v1/allOrders".to_owned(),
                Futures::UserTrades => "/fapi/v1/userTrades".to_owned(),
                Futures::PositionSide => "/fapi/v1/positionSide/dual".to_owned(),
                Futures::Order => "/fapi/v1/order".to_owned(),
                Futures::PositionRisk => "/fapi/v2/positionRisk".to_owned(),
                Futures::Balance => "/fapi/v2/balance".to_owned(),
                Futures::OpenInterest => "/fapi/v1/openInterest".to_owned(),
                Futures::OpenInterestHist => "/futures/data/openInterestHist".to_owned(),
                Futures::TopLongShortAccountRatio => {
                    "/futures/data/topLongShortAccountRatio".to_owned()
                }
                Futures::TopLongShortPositionRatio => {
                    "/futures/data/topLongShortPositionRatio".to_owned()
                }
                Futures::GlobalLongShortAccountRatio => {
                    "/futures/data/globalLongShortAccountRatio".to_owned()
                }
                Futures::TakerlongshortRatio => "/futures/data/takerlongshortRatio".to_owned(),
                Futures::LvtKlines => "/fapi/v1/lvtKlines".to_owned(),
                Futures::IndexInfo => "/fapi/v1/indexInfo".to_owned(),
                Futures::ChangeInitialLeverage => "/fapi/v1/leverage".to_owned(),
                Futures::Account => "/fapi/v2/account".to_owned(),
                Futures::OpenOrders => "/fapi/v1/openOrders".to_owned(),
                Futures::UserDataStream => "/fapi/v1/listenKey".to_owned(),
                Futures::Income => "/fapi/v1/income".to_owned(),
                Futures::HistoricalDataDownloadId => "/sapi/v1/futuresHistDataId".to_owned(),
                Futures::HistoricalDataDownloadLink => "/sapi/v1/downloadLink".to_owned(),
                Futures::DownloadLink(url) => url,
            },
        }
    }
}

pub trait Binance {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self;
    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> Self;
}

impl Binance for General {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> General {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> General {
        General {
            client: Client::new(api_key, secret_key, config.rest_api_endpoint.clone()),
        }
    }
}

impl Binance for Account {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Account {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> Account {
        Account {
            client: Client::new(api_key, secret_key, config.rest_api_endpoint.clone()),
            recv_window: config.recv_window,
        }
    }
}

impl Binance for Savings {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> Self {
        Self {
            client: Client::new(api_key, secret_key, config.rest_api_endpoint.clone()),
            recv_window: config.recv_window,
        }
    }
}

impl Binance for Market {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Market {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> Market {
        Market {
            client: Client::new(api_key, secret_key, config.rest_api_endpoint.clone()),
            recv_window: config.recv_window,
        }
    }
}

impl Binance for UserStream {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> UserStream {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> UserStream {
        UserStream {
            client: Client::new(api_key, secret_key, config.rest_api_endpoint.clone()),
            recv_window: config.recv_window,
        }
    }
}

// *****************************************************
//              Binance Futures API
// *****************************************************

impl Binance for FuturesGeneral {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> FuturesGeneral {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> FuturesGeneral {
        FuturesGeneral {
            client: Client::new(
                api_key,
                secret_key,
                config.futures_rest_api_endpoint.clone(),
            ),
        }
    }
}

impl Binance for FuturesMarket {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> FuturesMarket {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> FuturesMarket {
        FuturesMarket {
            client: Client::new(
                api_key,
                secret_key,
                config.futures_rest_api_endpoint.clone(),
            ),
            recv_window: config.recv_window,
        }
    }
}

impl Binance for FuturesAccount {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> Self {
        Self {
            client: Client::new(
                api_key,
                secret_key,
                config.futures_rest_api_endpoint.clone(),
            ),
            recv_window: config.recv_window,
        }
    }
}

impl Binance for FuturesUserStream {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> FuturesUserStream {
        Self::new_with_config(api_key, secret_key, &Config::default())
    }

    fn new_with_config(
        api_key: Option<String>, secret_key: Option<String>, config: &Config,
    ) -> FuturesUserStream {
        FuturesUserStream {
            client: Client::new(
                api_key,
                secret_key,
                config.futures_rest_api_endpoint.clone(),
            ),
            recv_window: config.recv_window,
        }
    }
}
