use rust_decimal::Decimal;
use std::str::FromStr;
use ucel_cex_binance::market_meta::parse_market_meta_snapshot_from_str;

#[test]
fn fixture_parses_and_normalizes() {
    let fixture = include_str!("../../../fixtures/market_meta/binance/spot_exchangeInfo.json");
    let snapshot = parse_market_meta_snapshot_from_str(fixture).expect("fixture should parse");

    assert_eq!(snapshot.markets.len(), 2);
    let btcusdt = snapshot
        .markets
        .iter()
        .find(|m| m.id.raw_symbol == "BTC/USDT")
        .expect("BTC/USDT market");

    assert_eq!(btcusdt.tick_size, Decimal::from_str("0.01").unwrap());
    assert_eq!(btcusdt.step_size, Decimal::from_str("0.00001").unwrap());
    assert_eq!(btcusdt.min_qty, Some(Decimal::from_str("0.00001").unwrap()));
    assert_eq!(btcusdt.max_qty, Some(Decimal::from_str("9000").unwrap()));
    assert_eq!(btcusdt.min_notional, Some(Decimal::from_str("10").unwrap()));
}
