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
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
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
    pub async fn connect(market: &FuturesMarket, subscription: &str) -> Result<Self> {
        Self::connect_wss(&FuturesWebsocketAPI::Default.params(market, subscription)).await
    }

    pub async fn connect_with_config(
        market: &FuturesMarket,
        subscription: &str,
        config: &Config,
    ) -> Result<Self> {
        Self::connect_wss(
            &FuturesWebsocketAPI::Custom(config.ws_endpoint.clone()).params(market, subscription),
        )
        .await
    }

    pub async fn connect_multiple_streams(
        market: &FuturesMarket,
        endpoints: &[String],
    ) -> Result<Self> {
        Self::connect_wss(&FuturesWebsocketAPI::MultiStream.params(market, &endpoints.join("/")))
            .await
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

    pub async fn recv(&mut self) -> Result<Option<FuturesWebsocketEvent>> {
        match self.read.next().await {
            Some(Ok(message)) => match message {
                Message::Text(msg) => Ok(Some(Self::handle_msg(&msg)?)),
                Message::Ping(_) => {
                    debug!("Ping received.");
                    self.write.send(Message::Pong(vec![])).await?;
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
