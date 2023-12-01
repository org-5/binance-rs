use binance::api::*;
use binance::config::*;
use binance::general::*;
use binance::model::*;

#[cfg(test)]
mod tests {
    use float_cmp::*;
    use mockito::mock;
    use tokio::test;

    use super::*;

    #[test]
    async fn ping() {
        let mock_ping = mock("GET", "/api/v3/ping")
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body("{}")
            .create();

        let config = Config::default().set_rest_api_endpoint(mockito::server_url());
        let general: General = Binance::new_with_config(None, None, &config).unwrap();

        let pong = general.ping().await.unwrap();
        mock_ping.assert();

        assert_eq!(pong, "pong");
    }

    #[test]
    async fn get_server_time() {
        let mock_server_time = mock("GET", "/api/v3/exchangeInfo")
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("tests/mocks/general/exchange_info.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(mockito::server_url());
        let mut general: General = Binance::new_with_config(None, None, &config).unwrap();
        general.update_cache().await.unwrap();

        let server_time = general.get_server_time().unwrap();
        mock_server_time.assert();

        assert_eq!(server_time.server_time, 1614694549948);
    }

    #[test]
    async fn exchange_info() {
        let mock_exchange_info = mock("GET", "/api/v3/exchangeInfo")
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("tests/mocks/general/exchange_info.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(mockito::server_url());
        let mut general: General = Binance::new_with_config(None, None, &config).unwrap();
        general.update_cache().await.unwrap();

        let exchange_info = general.exchange_info().unwrap().0;
        mock_exchange_info.assert();

        assert!(exchange_info.symbols.len() > 1);
    }

    #[test]
    async fn get_symbol_info() {
        let mock_exchange_info = mock("GET", "/api/v3/exchangeInfo")
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("tests/mocks/general/exchange_info.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(mockito::server_url());
        let mut general: General = Binance::new_with_config(None, None, &config).unwrap();
        general.update_cache().await.unwrap();

        let symbol = general.get_symbol_info("BNBBTC").unwrap();
        mock_exchange_info.assert();

        assert_eq!(symbol.symbol, "BNBBTC");
        assert_eq!(symbol.status, "TRADING");
        assert_eq!(symbol.base_asset, "BNB");
        assert_eq!(symbol.base_asset_precision, 8);
        assert_eq!(symbol.quote_asset, "BTC");
        assert_eq!(symbol.quote_precision, 8);

        assert!(!symbol.order_types.is_empty());
        assert_eq!(symbol.order_types[0], "LIMIT");
        assert_eq!(symbol.order_types[1], "LIMIT_MAKER");
        assert_eq!(symbol.order_types[2], "MARKET");
        assert_eq!(symbol.order_types[3], "STOP_LOSS_LIMIT");
        assert_eq!(symbol.order_types[4], "TAKE_PROFIT_LIMIT");

        assert!(symbol.iceberg_allowed);
        assert!(symbol.is_spot_trading_allowed);
        assert!(symbol.is_margin_trading_allowed);

        assert!(!symbol.filters.is_empty());

        for filter in symbol.filters.into_iter() {
            match filter {
                Filters::PriceFilter {
                    min_price,
                    max_price,
                    tick_size,
                } => {
                    assert_eq!(min_price, "0.00000010");
                    assert_eq!(max_price, "100000.00000000");
                    assert_eq!(tick_size, "0.00000010");
                }
                Filters::PercentPrice {
                    multiplier_up,
                    multiplier_down,
                    avg_price_mins,
                } => {
                    assert_eq!(multiplier_up, "5");
                    assert_eq!(multiplier_down, "0.2");
                    assert!(approx_eq!(f64, avg_price_mins.unwrap(), 5.0, ulps = 2));
                }
                Filters::LotSize {
                    min_qty,
                    max_qty,
                    step_size,
                } => {
                    assert_eq!(min_qty, "0.01000000");
                    assert_eq!(max_qty, "100000.00000000");
                    assert_eq!(step_size, "0.01000000");
                }
                Filters::MinNotional {
                    notional,
                    min_notional,
                    apply_to_market,
                    avg_price_mins,
                } => {
                    assert!(notional.is_none());
                    assert_eq!(min_notional.unwrap(), "0.00010000");
                    assert!(apply_to_market.unwrap());
                    assert!(approx_eq!(f64, avg_price_mins.unwrap(), 5.0, ulps = 2));
                }
                Filters::IcebergParts { limit } => {
                    assert_eq!(limit.unwrap(), 10);
                }
                Filters::MarketLotSize {
                    min_qty,
                    max_qty,
                    step_size,
                } => {
                    assert_eq!(min_qty, "0.00000000");
                    assert_eq!(max_qty, "8528.32329395");
                    assert_eq!(step_size, "0.00000000");
                }
                Filters::MaxNumOrders { max_num_orders } => {
                    assert_eq!(max_num_orders.unwrap(), 200);
                }
                Filters::MaxNumAlgoOrders {
                    max_num_algo_orders,
                } => {
                    assert_eq!(max_num_algo_orders.unwrap(), 5);
                }
                _ => panic!(),
            }
        }
    }
}
