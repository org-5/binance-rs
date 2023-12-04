use error_chain::bail;
use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::SinkExt;
use futures_util::StreamExt;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tracing::debug;
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
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
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
    pub async fn connect(subscription: &str) -> Result<Self> {
        Self::connect_wss(&WebsocketAPI::Default.params(subscription)).await
    }

    pub async fn connect_with_config(subscription: &str, config: &Config) -> Result<Self> {
        Self::connect_wss(&WebsocketAPI::Custom(config.ws_endpoint.clone()).params(subscription))
            .await
    }

    pub async fn connect_multiple_streams(endpoints: &[String]) -> Result<Self> {
        Self::connect_wss(&WebsocketAPI::MultiStream.params(&endpoints.join("/"))).await
    }

    async fn connect_wss(wss: &str) -> Result<Self> {
        let url = Url::parse(wss)?;
        match tokio_tungstenite::connect_async(url).await {
            Ok((socket, response)) => {
                debug!("Websocket handshake has been successfully completed");
                debug!("Response: {}", response.status());
                debug!("Response: {:?}", response.body());
                let (write, read) = socket.split();
                Ok(Self { write, read })
            }
            Err(e) => bail!(format!("Error during handshake {}", e)),
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.write.send(Message::Close(None)).await?;
        Ok(())
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

    pub async fn recv(&mut self) -> Result<Option<WebsocketEvent>> {
        match self.read.next().await {
            Some(Ok(message)) => match message {
                Message::Text(msg) => Ok(Some(Self::handle_msg(&msg)?)),
                Message::Ping(payload) => {
                    debug!("Ping received.");
                    self.write.send(Message::Pong(payload)).await?;
                    Ok(None)
                }
                Message::Pong(_) | Message::Binary(_) | Message::Frame(_) => Ok(None),
                Message::Close(e) => bail!(format!("Disconnected {:?}", e)),
            },
            Some(Err(e)) => Err(e.into()),
            None => {
                debug!("Websocket connection closed");
                Err("Websocket connection closed".into())
            }
        }
    }
}
