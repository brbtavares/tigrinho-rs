pub mod engine;
pub mod paytable;
pub mod rng;
pub mod symbols;

pub use crate::engine::{spin_once, spin_with_seeds, compute_reel_window, verify_reels, EngineParams, SpinOutcome};
pub use crate::paytable::{Paytable, PaytableEntry};
pub use crate::rng::{derive_floats, derive_hash_hex, ProvablyFairRng};
pub use crate::symbols::{ReelsConfig, Symbol};
