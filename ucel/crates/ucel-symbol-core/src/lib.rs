use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub mod market_meta;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exchange {
    Bitbank,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketType {
    Spot,
}

pub type InstrumentMeta = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotSource {
    Rest,
    Websocket,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotOrigin {
    pub source: SnapshotSource,
    pub restored: bool,
}

impl Default for SnapshotOrigin {
    fn default() -> Self {
        Self {
            source: SnapshotSource::Rest,
            restored: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StandardizedInstrument {
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub raw_symbol: String,
    pub base: String,
    pub quote: String,
    pub tick_size: Decimal,
    pub lot_size: Decimal,
    pub min_order_qty: Option<Decimal>,
    pub max_order_qty: Option<Decimal>,
    pub min_notional: Option<Decimal>,
    pub price_precision: Option<u32>,
    pub qty_precision: Option<u32>,
    pub contract_size: Option<Decimal>,
    #[serde(default)]
    pub meta: InstrumentMeta,
}

pub fn normalize_decimal(value: Decimal) -> Decimal {
    value.normalize()
}

pub fn cmp_decimal(a: Decimal, b: Decimal) -> Ordering {
    normalize_decimal(a).cmp(&normalize_decimal(b))
}

pub use market_meta::{
    MarketMeta, MarketMetaError, MarketMetaId, MarketMetaSnapshot, OrderSide, TickStepRounding,
    MARKET_META_SCHEMA_VERSION,
};
