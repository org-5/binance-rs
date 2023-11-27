use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use error_chain::bail;

use crate::api::Spot;
use crate::api::API;
use crate::client::Client;
use crate::errors::Result;
use crate::model::Empty;
use crate::model::ExchangeInformation;
use crate::model::ServerTime;
use crate::model::Symbol;

const CACHE_TTL: u64 = 600; // 10 minutes.

#[derive(Clone, Debug)]
pub struct General {
    pub client: Client,
    pub(crate) cache: Option<ExchangeInformation>,
    pub(crate) last_update: Option<u64>,
}

impl General {
    // Test connectivity
    pub fn ping(&self) -> Result<String> {
        self.client.get::<Empty>(API::Spot(Spot::Ping), None)?;
        Ok("pong".into())
    }

    // Check server time
    pub fn get_server_time(&self) -> Result<ServerTime> {
        if self.has_cache() {
            Ok(ServerTime {
                server_time: self.cache.as_ref().unwrap().server_time,
            })
        } else {
            Err("No cache".into())
        }
    }

    // Obtain exchange information
    // - Current exchange trading rules and symbol information
    // The boolean is true if the cache was used.
    pub fn exchange_info(&self) -> Result<(ExchangeInformation, bool)> {
        if self.has_cache() {
            Ok((self.cache.clone().unwrap(), true))
        } else {
            Err("No cache".into())
        }
    }

    pub fn update_cache(&mut self) -> Result<()> {
        let info: ExchangeInformation = self.client.get(API::Spot(Spot::ExchangeInfo), None)?;
        self.cache = Some(info.clone());
        self.last_update = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        Ok(())
    }

    pub fn has_cache(&self) -> bool {
        self.cache.is_some()
            && self.last_update.is_some()
            && SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - self.last_update.unwrap()
                < CACHE_TTL
    }

    // Get Symbol information
    pub fn get_symbol_info<S>(&mut self, symbol: S) -> Result<Symbol>
    where
        S: Into<String>,
    {
        let upper_symbol = symbol.into().to_uppercase();
        match self.exchange_info() {
            Ok(info) => {
                for item in info.0.symbols {
                    if item.symbol == upper_symbol {
                        return Ok(item);
                    }
                }
                bail!("Symbol not found")
            }
            Err(e) => Err(e),
        }
    }
}
