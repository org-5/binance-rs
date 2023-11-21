use std::net::TcpStream;

use error_chain::bail;
use serde::Deserialize;
use serde::Serialize;
use tracing::debug;
use tungstenite::connect;
use tungstenite::handshake::client::Response;
use tungstenite::protocol::WebSocket;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::Message;
use url::Url;

use crate::config::Config;
use crate::errors::Result;
use crate::futures::model;
use crate::model::AccountUpdateEvent;
use crate::model::AggrTradesEvent;
use crate::model::BookTickerEvent;
use crate::model::ContinuousKlineEvent;
use crate::model::DayTickerEvent;
use crate::model::DepthOrderBookEvent;
use crate::model::IndexKlineEvent;
use crate::model::IndexPriceEvent;
use crate::model::KlineEvent;
use crate::model::LiquidationEvent;
use crate::model::MarkPriceEvent;
use crate::model::MiniTickerEvent;
use crate::model::OrderBook;
use crate::model::TradeEvent;
use crate::model::UserDataStreamExpiredEvent;
#[allow(clippy::all)]
enum FuturesWebsocketAPI {
    Default,
    MultiStream,
    Custom(String),
}

pub enum FuturesMarket {
    USDM,
    COINM,
    Vanilla,
}

impl FuturesWebsocketAPI {
    fn params(self, market: &FuturesMarket, subscription: &str) -> String {
        let baseurl = match market {
            FuturesMarket::USDM => "wss://fstream.binance.com",
            FuturesMarket::COINM => "wss://dstream.binance.com",
            FuturesMarket::Vanilla => "wss://vstream.binance.com",
        };

        match self {
            FuturesWebsocketAPI::Default => {
                format!("{}/ws/{}", baseurl, subscription)
            }
            FuturesWebsocketAPI::MultiStream => {
                format!("{}/stream?streams={}", baseurl, subscription)
            }
            FuturesWebsocketAPI::Custom(url) => url,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FuturesWebsocketEvent {
    AccountUpdate(AccountUpdateEvent),
    OrderTrade(model::OrderTradeEvent),
    AggrTrades(AggrTradesEvent),
    Trade(TradeEvent),
    OrderBook(OrderBook),
    DayTicker(DayTickerEvent),
    MiniTicker(MiniTickerEvent),
    MiniTickerAll(Vec<MiniTickerEvent>),
    IndexPrice(IndexPriceEvent),
    MarkPrice(MarkPriceEvent),
    MarkPriceAll(Vec<MarkPriceEvent>),
    DayTickerAll(Vec<DayTickerEvent>),
    Kline(KlineEvent),
    ContinuousKline(ContinuousKlineEvent),
    IndexKline(IndexKlineEvent),
    Liquidation(LiquidationEvent),
    DepthOrderBook(DepthOrderBookEvent),
    BookTicker(BookTickerEvent),
    UserDataStreamExpiredEvent(UserDataStreamExpiredEvent),
}

pub struct FuturesWebSockets {
    pub socket: Option<(WebSocket<MaybeTlsStream<TcpStream>>, Response)>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum FuturesEvents {
    Vec(Vec<DayTickerEvent>),
    DayTickerEvent(DayTickerEvent),
    BookTickerEvent(BookTickerEvent),
    MiniTickerEvent(MiniTickerEvent),
    VecMiniTickerEvent(Vec<MiniTickerEvent>),
    AccountUpdateEvent(AccountUpdateEvent),
    OrderTradeEvent(model::OrderTradeEvent),
    AggrTradesEvent(AggrTradesEvent),
    IndexPriceEvent(IndexPriceEvent),
    MarkPriceEvent(MarkPriceEvent),
    VecMarkPriceEvent(Vec<MarkPriceEvent>),
    TradeEvent(TradeEvent),
    KlineEvent(KlineEvent),
    ContinuousKlineEvent(ContinuousKlineEvent),
    IndexKlineEvent(IndexKlineEvent),
    LiquidationEvent(LiquidationEvent),
    OrderBook(OrderBook),
    DepthOrderBookEvent(DepthOrderBookEvent),
    UserDataStreamExpiredEvent(UserDataStreamExpiredEvent),
}

impl FuturesWebSockets {
    pub fn new() -> FuturesWebSockets {
        FuturesWebSockets { socket: None }
    }

    pub fn connect(&mut self, market: &FuturesMarket, subscription: &str) -> Result<()> {
        self.connect_wss(&FuturesWebsocketAPI::Default.params(market, subscription))
    }

    pub fn connect_with_config(
        &mut self,
        market: &FuturesMarket,
        subscription: &str,
        config: &Config,
    ) -> Result<()> {
        self.connect_wss(
            &FuturesWebsocketAPI::Custom(config.ws_endpoint.clone()).params(market, subscription),
        )
    }

    pub fn connect_multiple_streams(
        &mut self,
        market: &FuturesMarket,
        endpoints: &[String],
    ) -> Result<()> {
        self.connect_wss(&FuturesWebsocketAPI::MultiStream.params(market, &endpoints.join("/")))
    }

    fn connect_wss(&mut self, wss: &str) -> Result<()> {
        let url = Url::parse(wss)?;
        match connect(url) {
            Ok(answer) => {
                self.socket = Some(answer);
                Ok(())
            }
            Err(e) => bail!(format!("Error during handshake {}", e)),
        }
    }

    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(ref mut socket) = self.socket {
            socket.0.close(None)?;
            return Ok(());
        }
        bail!("Not able to close the connection");
    }

    fn handle_msg(msg: &str) -> Result<FuturesWebsocketEvent> {
        let value: serde_json::Value = serde_json::from_str(msg)?;

        if let Some(data) = value.get("data") {
            return Self::handle_msg(&data.to_string());
        }

        let events = serde_json::from_value::<FuturesEvents>(value)?;
        let events = match events {
            FuturesEvents::Vec(v) => FuturesWebsocketEvent::DayTickerAll(v),
            FuturesEvents::DayTickerEvent(v) => FuturesWebsocketEvent::DayTicker(v),
            FuturesEvents::BookTickerEvent(v) => FuturesWebsocketEvent::BookTicker(v),
            FuturesEvents::MiniTickerEvent(v) => FuturesWebsocketEvent::MiniTicker(v),
            FuturesEvents::VecMiniTickerEvent(v) => FuturesWebsocketEvent::MiniTickerAll(v),
            FuturesEvents::AccountUpdateEvent(v) => FuturesWebsocketEvent::AccountUpdate(v),
            FuturesEvents::OrderTradeEvent(v) => FuturesWebsocketEvent::OrderTrade(v),
            FuturesEvents::IndexPriceEvent(v) => FuturesWebsocketEvent::IndexPrice(v),
            FuturesEvents::MarkPriceEvent(v) => FuturesWebsocketEvent::MarkPrice(v),
            FuturesEvents::VecMarkPriceEvent(v) => FuturesWebsocketEvent::MarkPriceAll(v),
            FuturesEvents::TradeEvent(v) => FuturesWebsocketEvent::Trade(v),
            FuturesEvents::ContinuousKlineEvent(v) => FuturesWebsocketEvent::ContinuousKline(v),
            FuturesEvents::IndexKlineEvent(v) => FuturesWebsocketEvent::IndexKline(v),
            FuturesEvents::LiquidationEvent(v) => FuturesWebsocketEvent::Liquidation(v),
            FuturesEvents::KlineEvent(v) => FuturesWebsocketEvent::Kline(v),
            FuturesEvents::OrderBook(v) => FuturesWebsocketEvent::OrderBook(v),
            FuturesEvents::DepthOrderBookEvent(v) => FuturesWebsocketEvent::DepthOrderBook(v),
            FuturesEvents::AggrTradesEvent(v) => FuturesWebsocketEvent::AggrTrades(v),
            FuturesEvents::UserDataStreamExpiredEvent(v) => {
                FuturesWebsocketEvent::UserDataStreamExpiredEvent(v)
            }
        };
        Ok(events)
    }

    pub fn recv(&mut self) -> Result<Option<FuturesWebsocketEvent>> {
        if let Some(ref mut socket) = self.socket {
            let message = socket.0.read_message()?;
            match message {
                Message::Text(msg) => Ok(Some(Self::handle_msg(&msg)?)),
                Message::Ping(_) => {
                    debug!("Ping received.");
                    socket.0.write_message(Message::Pong(vec![]))?;
                    Ok(None)
                }
                Message::Pong(_) | Message::Binary(_) | Message::Frame(_) => Ok(None),
                Message::Close(e) => bail!(format!("Disconnected {:?}", e)),
            }
        } else {
            bail!("Websocket connection not initialized")
        }
    }
}

impl Default for FuturesWebSockets {
    fn default() -> Self {
        Self::new()
    }
}
