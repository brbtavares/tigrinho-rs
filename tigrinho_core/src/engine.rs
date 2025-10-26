use crate::{
    paytable::{Paytable, PaytableEntry},
    rng::ProvablyFairRng,
    symbols::{ReelsConfig, Symbol},
};

#[derive(Debug, Clone)]
pub struct EngineParams {
    pub reels: ReelsConfig,
    pub paytable: Paytable,
    pub rtp_target: f64, // not fully used; placeholder for future balancing
}

#[derive(Debug, Clone)]
pub struct SpinOutcome {
    pub reel_window: Vec<Vec<Symbol>>, // rows x cols symbols
    pub payout: f64,
}

pub fn compute_reel_window(rng: &ProvablyFairRng, reels: &ReelsConfig) -> Vec<Vec<Symbol>> {
    // For each reel, pick a starting index from RNG floats and take `rows` symbols circularly
    let cols = reels.reels.len();
    let floats = rng.next_floats(cols);
    let mut window: Vec<Vec<Symbol>> = vec![vec![Symbol::A; cols]; reels.rows];
    for (col, reel) in reels.reels.iter().enumerate() {
        let start = ((floats[col] * reel.len() as f64).floor() as usize) % reel.len();
        for r in 0..reels.rows {
            let idx = (start + r) % reel.len();
            window[r][col] = reel[idx];
        }
    }
    window
}

fn evaluate_payout(window: &Vec<Vec<Symbol>>, paytable: &Paytable, bet: f64) -> f64 {
    // Super-simple rule: pay only for row-wise 3-in-a-row of same symbol (Wild matches any)
    let rows = window.len();
    let mut total = 0.0;
    for r in 0..rows {
        let a = window[r][0];
        let b = window[r][1];
        let c = window[r][2];
        // count matches treating wild as joker
        let all_same = match (a, b, c) {
            (Symbol::Wild, _, _) | (_, Symbol::Wild, _) | (_, _, Symbol::Wild) => true,
            _ => a == b && b == c,
        };
        if all_same {
            let sym = if a == Symbol::Wild { b } else { a };
            if let Some(PaytableEntry {
                payout_multiplier, ..
            }) = paytable
                .0
                .iter()
                .find(|e| e.symbol == sym.to_index() && e.count == 3)
            {
                total += bet * payout_multiplier;
            }
        }
    }
    total
}

pub fn spin_once(
    rng: &ProvablyFairRng,
    params: &EngineParams,
    bet: f64,
    _lines: u32,
) -> SpinOutcome {
            let window = compute_reel_window(rng, &params.reels);
    let payout = evaluate_payout(&window, &params.paytable, bet);
    SpinOutcome {
        reel_window: window,
        payout,
    }
}

/// Convenience: perform a spin creating the RNG from seeds.
pub fn spin_with_seeds(
    server_seed: &str,
    client_seed: &str,
    nonce: u64,
    params: &EngineParams,
    bet: f64,
    lines: u32,
) -> SpinOutcome {
    let rng = ProvablyFairRng::new(server_seed, client_seed, nonce);
    spin_once(&rng, params, bet, lines)
}

/// Verify that a given reel window matches what the RNG would produce for the seeds.
pub fn verify_reels(
    server_seed: &str,
    client_seed: &str,
    nonce: u64,
    reels: &ReelsConfig,
    expected_indices: &[Vec<u8>],
) -> bool {
    let rng = ProvablyFairRng::new(server_seed, client_seed, nonce);
    let window = compute_reel_window(&rng, reels);
    let actual_indices: Vec<Vec<u8>> = window
        .iter()
        .map(|row| row.iter().map(|s| s.to_index()).collect())
        .collect();
    actual_indices == expected_indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spin_deterministic() {
        let params = EngineParams {
            reels: ReelsConfig::default_3x3(),
            paytable: Paytable::simple_default(),
            rtp_target: 0.95,
        };
        let rng = ProvablyFairRng::new("server", "client", 1);
        let out1 = spin_once(&rng, &params, 1.0, 1);
        let out2 = spin_once(&rng, &params, 1.0, 1);
        assert_eq!(out1.payout, out2.payout);
        assert_eq!(out1.reel_window, out2.reel_window);
    }
}
