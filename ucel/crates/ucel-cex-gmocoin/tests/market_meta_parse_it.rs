use rust_decimal::Decimal;
use std::str::FromStr;
use ucel_cex_gmocoin::market_meta::parse_market_meta_snapshot_from_str;

#[test]
fn fixture_parses_and_validates() {
    let fixture = include_str!("../../../fixtures/market_meta/gmocoin/public_v1_symbols.json");
    let snapshot = parse_market_meta_snapshot_from_str(fixture).expect("fixture should parse");

    assert_eq!(snapshot.markets.len(), 2);
    let btc = snapshot
        .markets
        .iter()
        .find(|m| m.id.raw_symbol == "BTC")
        .expect("BTC market");

    assert_eq!(btc.tick_size, Decimal::from_str("1").unwrap());
    assert_eq!(btc.step_size, Decimal::from_str("0.0001").unwrap());
    assert_eq!(btc.min_qty, Some(Decimal::from_str("0.0001").unwrap()));
    assert_eq!(btc.max_qty, Some(Decimal::from_str("5").unwrap()));
}
