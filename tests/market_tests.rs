use binance::config::*;
use binance::model::*;
use binance::spot::market::*;

#[cfg(test)]
mod tests {
    use binance::spot::model::Prices;
    use float_cmp::*;
    use mockito::Matcher;
    use rust_decimal::prelude::FromPrimitive;
    use tokio::test;

    use super::*;

    #[test]
    async fn get_depth() {
        let mut server = mockito::Server::new_async().await;
        let mock_get_depth = server
            .mock("GET", "/api/v3/depth")
            .with_header("content-type", "application/json;charset=UTF-8")
            .match_query(Matcher::Regex("symbol=LTCBTC".into()))
            .with_body_from_file("tests/mocks/market/get_depth.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(server.url());
        let market = Market::new_with_config(None, None, &config).unwrap();

        let order_book = market.get_depth("LTCBTC").await.unwrap();
        mock_get_depth.assert();

        assert_eq!(order_book.last_update_id, 1_027_024);
        assert_eq!(
            order_book.bids[0],
            Bids::new(
                rust_decimal::Decimal::from_f64(4.000_000_00).unwrap(),
                rust_decimal::Decimal::from_f64(431.000_000_00).unwrap()
            )
        );
    }

    #[test]
    async fn get_custom_depth() {
        let mut server = mockito::Server::new_async().await;
        let mock_get_custom_depth = server
            .mock("GET", "/api/v3/depth")
            .with_header("content-type", "application/json;charset=UTF-8")
            .match_query(Matcher::Regex("limit=10&symbol=LTCBTC".into()))
            .with_body_from_file("tests/mocks/market/get_depth.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(server.url());
        let market = Market::new_with_config(None, None, &config).unwrap();

        let order_book = market.get_custom_depth("LTCBTC", 10).await.unwrap();
        mock_get_custom_depth.assert();

        assert_eq!(order_book.last_update_id, 1_027_024);
        assert_eq!(
            order_book.bids[0],
            Bids::new(
                rust_decimal::Decimal::from_f64(4.000_000_00).unwrap(),
                rust_decimal::Decimal::from_f64(431.000_000_00).unwrap()
            )
        );
    }

    #[test]
    async fn get_all_prices() {
        let mut server = mockito::Server::new_async().await;
        let mock_get_all_prices = server
            .mock("GET", "/api/v3/ticker/price")
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("tests/mocks/market/get_all_prices.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(server.url());
        let market = Market::new_with_config(None, None, &config).unwrap();

        let prices = market.get_all_prices().await.unwrap();
        mock_get_all_prices.assert();

        match prices {
            Prices::AllPrices(symbols) => {
                assert!(!symbols.is_empty());
                let first_symbol = symbols[0].clone();
                assert_eq!(first_symbol.symbol, "LTCBTC");
                assert!(approx_eq!(f64, first_symbol.price, 4.000_002_00, ulps = 2));
                let second_symbol = symbols[1].clone();
                assert_eq!(second_symbol.symbol, "ETHBTC");
                assert!(approx_eq!(f64, second_symbol.price, 0.079_466_00, ulps = 2));
            }
        }
    }

    #[test]
    async fn get_price() {
        let mut server = mockito::Server::new_async().await;
        let mock_get_price = server
            .mock("GET", "/api/v3/ticker/price")
            .with_header("content-type", "application/json;charset=UTF-8")
            .match_query(Matcher::Regex("symbol=LTCBTC".into()))
            .with_body_from_file("tests/mocks/market/get_price.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(server.url());
        let market = Market::new_with_config(None, None, &config).unwrap();

        let symbol = market.get_price("LTCBTC").await.unwrap();
        mock_get_price.assert();

        assert_eq!(symbol.symbol, "LTCBTC");
        assert!(approx_eq!(f64, symbol.price, 4.000_002_00, ulps = 2));
    }

    #[test]
    async fn get_average_price() {
        let mut server = mockito::Server::new_async().await;
        let mock_get_average_price = server
            .mock("GET", "/api/v3/avgPrice")
            .with_header("content-type", "application/json;charset=UTF-8")
            .match_query(Matcher::Regex("symbol=LTCBTC".into()))
            .with_body_from_file("tests/mocks/market/get_average_price.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(server.url());
        let market = Market::new_with_config(None, None, &config).unwrap();

        let symbol = market.get_average_price("LTCBTC").await.unwrap();
        mock_get_average_price.assert();

        assert_eq!(symbol.mins, 5);
        assert!(approx_eq!(f64, symbol.price, 9.357_518_34, ulps = 2));
    }

    #[test]
    async fn get_all_book_tickers() {
        let mut server = mockito::Server::new_async().await;
        let mock_get_all_book_tickers = server
            .mock("GET", "/api/v3/ticker/bookTicker")
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("tests/mocks/market/get_all_book_tickers.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(server.url());
        let market = Market::new_with_config(None, None, &config).unwrap();

        let book_tickers = market.get_all_book_tickers().await.unwrap();
        mock_get_all_book_tickers.assert();

        match book_tickers {
            binance::model::BookTickers::AllBookTickers(tickers) => {
                assert!(!tickers.is_empty());
                let first_ticker = tickers[0].clone();
                assert_eq!(first_ticker.symbol, "LTCBTC");
                assert!(approx_eq!(
                    f64,
                    first_ticker.bid_price,
                    4.000_000_00,
                    ulps = 2
                ));
                assert!(approx_eq!(
                    f64,
                    first_ticker.bid_qty,
                    431.000_000_00,
                    ulps = 2
                ));
                assert!(approx_eq!(
                    f64,
                    first_ticker.ask_price,
                    4.000_002_00,
                    ulps = 2
                ));
                assert!(approx_eq!(
                    f64,
                    first_ticker.ask_qty,
                    9.000_000_00,
                    ulps = 2
                ));
                let second_ticker = tickers[1].clone();
                assert_eq!(second_ticker.symbol, "ETHBTC");
                assert!(approx_eq!(
                    f64,
                    second_ticker.bid_price,
                    0.079_467_00,
                    ulps = 2
                ));
                assert!(approx_eq!(
                    f64,
                    second_ticker.bid_qty,
                    9.000_000_00,
                    ulps = 2
                ));
                assert!(approx_eq!(
                    f64,
                    second_ticker.ask_price,
                    100_000.000_000_00,
                    ulps = 2
                ));
                assert!(approx_eq!(
                    f64,
                    second_ticker.ask_qty,
                    1_000.000_000_00,
                    ulps = 2
                ));
            }
        }
    }

    #[test]
    async fn get_book_ticker() {
        let mut server = mockito::Server::new_async().await;
        let mock_get_book_ticker = server
            .mock("GET", "/api/v3/ticker/bookTicker")
            .with_header("content-type", "application/json;charset=UTF-8")
            .match_query(Matcher::Regex("symbol=LTCBTC".into()))
            .with_body_from_file("tests/mocks/market/get_book_ticker.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(server.url());
        let market = Market::new_with_config(None, None, &config).unwrap();

        let book_ticker = market.get_book_ticker("LTCBTC").await.unwrap();
        mock_get_book_ticker.assert();

        assert_eq!(book_ticker.symbol, "LTCBTC");
        assert!(approx_eq!(
            f64,
            book_ticker.bid_price,
            4.000_000_00,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            book_ticker.bid_qty,
            431.000_000_00,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            book_ticker.ask_price,
            4.000_002_00,
            ulps = 2
        ));
        assert!(approx_eq!(f64, book_ticker.ask_qty, 9.000_000_00, ulps = 2));
    }

    #[test]
    async fn get_24h_price_stats() {
        let mut server = mockito::Server::new_async().await;
        let mock_get_24h_price_stats = server
            .mock("GET", "/api/v3/ticker/24hr")
            .with_header("content-type", "application/json;charset=UTF-8")
            .match_query(Matcher::Regex("symbol=BNBBTC".into()))
            .with_body_from_file("tests/mocks/market/get_24h_price_stats.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(server.url());
        let market = Market::new_with_config(None, None, &config).unwrap();

        let price_stats = market.get_24h_price_stats("BNBBTC").await.unwrap();
        mock_get_24h_price_stats.assert();

        assert_eq!(price_stats.symbol, "BNBBTC");
        assert_eq!(price_stats.price_change, "-94.99999800");
        assert_eq!(price_stats.price_change_percent, "-95.960");
        assert_eq!(price_stats.weighted_avg_price, "0.29628482");
        assert!(approx_eq!(
            f64,
            price_stats.prev_close_price,
            0.100_020_00,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            price_stats.last_price,
            4.000_002_00,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            price_stats.bid_price,
            4.000_000_00,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            price_stats.ask_price,
            4.000_002_00,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            price_stats.open_price,
            99.000_000_00,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            price_stats.high_price,
            100.000_000_00,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            price_stats.low_price,
            0.100_000_00,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            price_stats.volume,
            8_913.300_000_00,
            ulps = 2
        ));
        assert_eq!(price_stats.open_time, 1_499_783_499_040);
        assert_eq!(price_stats.close_time, 1_499_869_899_040);
        assert_eq!(price_stats.first_id, 28385);
        assert_eq!(price_stats.last_id, 28460);
        assert_eq!(price_stats.count, 76);
    }

    #[test]
    async fn get_all_24h_price_stats() {
        let mut server = mockito::Server::new_async().await;
        let mock_get_all_24h_price_stats = server
            .mock("GET", "/api/v3/ticker/24hr")
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("tests/mocks/market/get_all_24h_price_stats.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(server.url());
        let market = Market::new_with_config(None, None, &config).unwrap();

        let prices_stats = market.get_all_24h_price_stats().await.unwrap();
        mock_get_all_24h_price_stats.assert();

        assert!(!prices_stats.is_empty());

        let ps = prices_stats[0].clone();

        assert_eq!(ps.symbol, "BNBBTC");
        assert_eq!(ps.price_change, "-94.99999800");
        assert_eq!(ps.price_change_percent, "-95.960");
        assert_eq!(ps.weighted_avg_price, "0.29628482");
        assert!(approx_eq!(f64, ps.prev_close_price, 0.100_020_00, ulps = 2));
        assert!(approx_eq!(f64, ps.last_price, 4.000_002_00, ulps = 2));
        assert!(approx_eq!(f64, ps.bid_price, 4.000_000_00, ulps = 2));
        assert!(approx_eq!(f64, ps.ask_price, 4.000_002_00, ulps = 2));
        assert!(approx_eq!(f64, ps.open_price, 99.000_000_00, ulps = 2));
        assert!(approx_eq!(f64, ps.high_price, 100.000_000_00, ulps = 2));
        assert!(approx_eq!(f64, ps.low_price, 0.100_000_00, ulps = 2));
        assert!(approx_eq!(f64, ps.volume, 8_913.300_000_00, ulps = 2));
        assert_eq!(ps.open_time, 1_499_783_499_040);
        assert_eq!(ps.close_time, 1_499_869_899_040);
        assert_eq!(ps.first_id, 28385);
        assert_eq!(ps.last_id, 28460);
        assert_eq!(ps.count, 76);
    }

    #[test]
    async fn get_klines() {
        let mut server = mockito::Server::new_async().await;
        let mock_get_klines = server
            .mock("GET", "/api/v3/klines")
            .with_header("content-type", "application/json;charset=UTF-8")
            .match_query(Matcher::Regex("interval=5m&limit=10&symbol=LTCBTC".into()))
            .with_body_from_file("tests/mocks/market/get_klines.json")
            .create();

        let config = Config::default().set_rest_api_endpoint(server.url());
        let market = Market::new_with_config(None, None, &config).unwrap();

        let klines = market
            .get_klines("LTCBTC", "5m", 10, None, None)
            .await
            .unwrap();
        mock_get_klines.assert();

        match klines {
            binance::model::KlineSummaries::AllKlineSummaries(klines) => {
                assert!(!klines.is_empty());
                let kline: KlineSummary = klines[0].clone();

                assert_eq!(kline.open_time, 1_499_040_000_000);
                assert_eq!(kline.open, "0.01634790");
                assert_eq!(kline.high, "0.80000000");
                assert_eq!(kline.low, "0.01575800");
                assert_eq!(kline.close, "0.01577100");
                assert_eq!(kline.volume, "148976.11427815");
                assert_eq!(kline.close_time, 1_499_644_799_999);
                assert_eq!(kline.quote_asset_volume, "2434.19055334");
                assert_eq!(kline.number_of_trades, 308);
                assert_eq!(kline.taker_buy_base_asset_volume, "1756.87402397");
                assert_eq!(kline.taker_buy_quote_asset_volume, "28.46694368");
            }
        }
    }
}
