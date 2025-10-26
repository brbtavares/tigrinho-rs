use crate::symbols::Symbol;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaytableEntry {
    pub symbol: u8, // Symbol index
    pub count: u8,
    pub payout_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paytable(pub Vec<PaytableEntry>);

impl Paytable {
    pub fn simple_default() -> Self {
        Self(vec![
            PaytableEntry {
                symbol: Symbol::A.to_index(),
                count: 3,
                payout_multiplier: 5.0,
            },
            PaytableEntry {
                symbol: Symbol::B.to_index(),
                count: 3,
                payout_multiplier: 4.0,
            },
            PaytableEntry {
                symbol: Symbol::C.to_index(),
                count: 3,
                payout_multiplier: 3.0,
            },
            PaytableEntry {
                symbol: Symbol::D.to_index(),
                count: 3,
                payout_multiplier: 2.0,
            },
            PaytableEntry {
                symbol: Symbol::Wild.to_index(),
                count: 3,
                payout_multiplier: 10.0,
            },
        ])
    }
}
