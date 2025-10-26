use tigrinho_core::{EngineParams, Paytable, ProvablyFairRng, ReelsConfig};

#[test]
fn rng_repeatable() {
    let rng1 = ProvablyFairRng::new("s", "c", 42);
    let rng2 = ProvablyFairRng::new("s", "c", 42);
    assert_eq!(rng1.next_floats(10), rng2.next_floats(10));
}

#[test]
fn payout_basic() {
    let params = EngineParams {
        reels: ReelsConfig::default_3x3(),
        paytable: Paytable::simple_default(),
        rtp_target: 0.95,
    };
    let rng = ProvablyFairRng::new("server", "client", 7);
    let out = tigrinho_core::engine::spin_once(&rng, &params, 1.0, 1);
    assert!(out.payout >= 0.0);
}

#[test]
fn rtp_simulation_smoke() {
    let params = EngineParams {
        reels: ReelsConfig::default_3x3(),
        paytable: Paytable::simple_default(),
        rtp_target: 0.95,
    };
    let mut total_bet = 0.0;
    let mut total_payout = 0.0;
    for n in 0..1000u64 {
        let rng = ProvablyFairRng::new("server", "client", n);
        let out = tigrinho_core::engine::spin_once(&rng, &params, 1.0, 1);
        total_bet += 1.0;
        total_payout += out.payout;
    }
    let rtp = total_payout / total_bet;
    // very loose bounds since default table is arbitrary
    assert!(rtp >= 0.0 && rtp <= 10.0);
}
