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
use crate::model::AccountUpdateEvent;
use crate::model::AggrTradesEvent;
use crate::model::BalanceUpdateEvent;
use crate::model::BookTickerEvent;
use crate::model::DayTickerEvent;
use crate::model::DepthOrderBookEvent;
use crate::model::KlineEvent;
use crate::model::OrderBook;
use crate::model::OrderTradeEvent;
use crate::model::TradeEvent;

#[allow(clippy::all)]
enum WebsocketAPI {
    Default,
    MultiStream,
    Custom(String),
}

impl WebsocketAPI {
    fn params(self, subscription: &str) -> String {
        match self {
            WebsocketAPI::Default => format!("wss://stream.binance.com:9443/ws/{}", subscription),
            WebsocketAPI::MultiStream => format!(
                "wss://stream.binance.com:9443/stream?streams={}",
                subscription
            ),
            WebsocketAPI::Custom(url) => format!("{}/{}", url, subscription),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WebsocketEvent {
    AccountUpdate(AccountUpdateEvent),
    BalanceUpdate(BalanceUpdateEvent),
    OrderTrade(OrderTradeEvent),
    AggrTrades(AggrTradesEvent),
    Trade(TradeEvent),
    OrderBook(OrderBook),
    DayTicker(DayTickerEvent),
    DayTickerAll(Vec<DayTickerEvent>),
    Kline(KlineEvent),
    DepthOrderBook(DepthOrderBookEvent),
    BookTicker(BookTickerEvent),
}

pub struct WebSockets {
    pub socket: Option<(WebSocket<MaybeTlsStream<TcpStream>>, Response)>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Events {
    Vec(Vec<DayTickerEvent>),
    BalanceUpdateEvent(BalanceUpdateEvent),
    DayTickerEvent(DayTickerEvent),
    BookTickerEvent(BookTickerEvent),
    AccountUpdateEvent(AccountUpdateEvent),
    OrderTradeEvent(OrderTradeEvent),
    AggrTradesEvent(AggrTradesEvent),
    TradeEvent(TradeEvent),
    KlineEvent(KlineEvent),
    OrderBook(OrderBook),
    DepthOrderBookEvent(DepthOrderBookEvent),
}

impl WebSockets {
    pub fn new() -> WebSockets {
        WebSockets { socket: None }
    }

    pub fn connect(&mut self, subscription: &str) -> Result<()> {
        self.connect_wss(&WebsocketAPI::Default.params(subscription))
    }

    pub fn connect_with_config(&mut self, subscription: &str, config: &Config) -> Result<()> {
        self.connect_wss(&WebsocketAPI::Custom(config.ws_endpoint.clone()).params(subscription))
    }

    pub fn connect_multiple_streams(&mut self, endpoints: &[String]) -> Result<()> {
        self.connect_wss(&WebsocketAPI::MultiStream.params(&endpoints.join("/")))
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

    fn handle_msg(msg: &str) -> Result<WebsocketEvent> {
        let value: serde_json::Value = serde_json::from_str(msg)?;

        if let Some(data) = value.get("data") {
            return Self::handle_msg(&data.to_string());
        }

        let events = serde_json::from_value::<Events>(value)?;
        let events = match events {
            Events::Vec(v) => WebsocketEvent::DayTickerAll(v),
            Events::BookTickerEvent(v) => WebsocketEvent::BookTicker(v),
            Events::BalanceUpdateEvent(v) => WebsocketEvent::BalanceUpdate(v),
            Events::AccountUpdateEvent(v) => WebsocketEvent::AccountUpdate(v),
            Events::OrderTradeEvent(v) => WebsocketEvent::OrderTrade(v),
            Events::AggrTradesEvent(v) => WebsocketEvent::AggrTrades(v),
            Events::TradeEvent(v) => WebsocketEvent::Trade(v),
            Events::DayTickerEvent(v) => WebsocketEvent::DayTicker(v),
            Events::KlineEvent(v) => WebsocketEvent::Kline(v),
            Events::OrderBook(v) => WebsocketEvent::OrderBook(v),
            Events::DepthOrderBookEvent(v) => WebsocketEvent::DepthOrderBook(v),
        };
        Ok(events)
    }

    pub fn recv(&mut self) -> Result<Option<WebsocketEvent>> {
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

impl Default for WebSockets {
    fn default() -> Self {
        Self::new()
    }
}
