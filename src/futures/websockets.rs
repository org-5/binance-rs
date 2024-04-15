use error_chain::bail;
use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::SinkExt;
use futures_util::StreamExt;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tracing::debug;
use url::Url;

use super::model::OrderBook;
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
use crate::model::TradeEvent;
use crate::model::UserDataStreamExpiredEvent;

enum WebsocketsApi {
    Default,
    MultiStream,
    Custom(String),
}

pub enum FuturesMarket {
    USDM,
    COINM,
    Vanilla,
}

impl WebsocketsApi {
    fn params(self, market: &FuturesMarket, subscription: &str) -> String {
        let baseurl = match market {
            FuturesMarket::USDM => "wss://fstream.binance.com",
            FuturesMarket::COINM => "wss://dstream.binance.com",
            FuturesMarket::Vanilla => "wss://vstream.binance.com",
        };

        match self {
            WebsocketsApi::Default => {
                format!("{baseurl}/ws/{subscription}")
            }
            WebsocketsApi::MultiStream => {
                format!("{baseurl}/stream?streams={subscription}")
            }
            WebsocketsApi::Custom(url) => url,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WebsocketEvent {
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

pub struct WebSockets {
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Events {
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

impl WebSockets {
    /// Connect to the Binance Websocket API.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    pub async fn connect(market: &FuturesMarket, subscription: &str) -> Result<Self> {
        Self::connect_wss(&WebsocketsApi::Default.params(market, subscription)).await
    }

    /// Connect to the Binance Websocket API with a custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    pub async fn connect_with_config(
        market: &FuturesMarket,
        subscription: &str,
        config: &Config,
    ) -> Result<Self> {
        Self::connect_wss(
            &WebsocketsApi::Custom(config.ws_endpoint.clone()).params(market, subscription),
        )
        .await
    }

    /// Connect to the Binance Websocket API with multiple streams.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    pub async fn connect_multiple_streams(
        market: &FuturesMarket,
        endpoints: &[String],
    ) -> Result<Self> {
        Self::connect_wss(&WebsocketsApi::MultiStream.params(market, &endpoints.join("/"))).await
    }

    async fn connect_wss(wss: &str) -> Result<Self> {
        let url = Url::parse(wss)?;
        match tokio_tungstenite::connect_async(url).await {
            Ok((socket, response)) => {
                debug!("Websocket handshake has been successfully completed");
                debug!("Response: {}", response.status());
                debug!("Response: {:?}", response.body());
                let (write, read) = socket.split();
                Ok(Self { read, write })
            }
            Err(e) => bail!(format!("Error during handshake {}", e)),
        }
    }

    /// Disconnect from the Binance Websocket API.
    ///
    /// # Errors
    ///
    /// Returns an error if the disconnection fails.
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
            Events::DayTickerEvent(v) => WebsocketEvent::DayTicker(v),
            Events::BookTickerEvent(v) => WebsocketEvent::BookTicker(v),
            Events::MiniTickerEvent(v) => WebsocketEvent::MiniTicker(v),
            Events::VecMiniTickerEvent(v) => WebsocketEvent::MiniTickerAll(v),
            Events::AccountUpdateEvent(v) => WebsocketEvent::AccountUpdate(v),
            Events::OrderTradeEvent(v) => WebsocketEvent::OrderTrade(v),
            Events::IndexPriceEvent(v) => WebsocketEvent::IndexPrice(v),
            Events::MarkPriceEvent(v) => WebsocketEvent::MarkPrice(v),
            Events::VecMarkPriceEvent(v) => WebsocketEvent::MarkPriceAll(v),
            Events::TradeEvent(v) => WebsocketEvent::Trade(v),
            Events::ContinuousKlineEvent(v) => WebsocketEvent::ContinuousKline(v),
            Events::IndexKlineEvent(v) => WebsocketEvent::IndexKline(v),
            Events::LiquidationEvent(v) => WebsocketEvent::Liquidation(v),
            Events::KlineEvent(v) => WebsocketEvent::Kline(v),
            Events::OrderBook(v) => WebsocketEvent::OrderBook(v),
            Events::DepthOrderBookEvent(v) => WebsocketEvent::DepthOrderBook(v),
            Events::AggrTradesEvent(v) => WebsocketEvent::AggrTrades(v),
            Events::UserDataStreamExpiredEvent(v) => WebsocketEvent::UserDataStreamExpiredEvent(v),
        };
        Ok(events)
    }

    /// Receive a message from the Binance Websocket API.
    ///
    /// # Errors
    ///
    /// Returns an error if the message fails to be received.
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
