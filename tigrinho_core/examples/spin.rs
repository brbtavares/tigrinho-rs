use tigrinho_core::{spin_once, EngineParams, Paytable, ProvablyFairRng, ReelsConfig};

fn main() {
    // Example end-to-end spin
    let server_seed = "example-server-seed";
    let client_seed = "example-client-seed";
    let nonce = 1u64;
    let rng = ProvablyFairRng::new(server_seed, client_seed, nonce);
    let params = EngineParams {
        reels: ReelsConfig::default_3x3(),
        paytable: Paytable::simple_default(),
        rtp_target: 0.95,
    };
    let outcome = spin_once(&rng, &params, 1.0, 1);
    println!(
        "server_seed_hash={} payout={} window={:?}",
        rng.server_seed_hash_hex(),
        outcome.payout,
        outcome.reel_window
    );
}
