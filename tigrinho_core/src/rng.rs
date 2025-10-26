use hmac::{Hmac, Mac};
use sha2::Sha256;

// Deterministic RNG using provably-fair HMAC construction
// server_seed (secret) + client_seed + nonce -> HMAC-SHA256 -> bytes -> floats in [0,1)

pub type HmacSha256 = Hmac<Sha256>;

pub fn derive_hash_hex(input: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(input);
    hex::encode(hasher.finalize())
}

pub fn derive_floats(hmac_bytes: &[u8], count: usize) -> Vec<f64> {
    // Convert successive 4-byte chunks into u32 then map to [0,1)
    let mut out = Vec::with_capacity(count);
    let mut buffer = hmac_bytes.to_vec();
    let mut i = 0usize;
    while out.len() < count {
        if i + 4 > buffer.len() {
            // extend the buffer deterministically by hashing the previous buffer
            let hex = derive_hash_hex(&buffer);
            buffer = hex::decode(hex).expect("valid hex");
            i = 0;
            continue;
        }
        let chunk = &buffer[i..i + 4];
        let v = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let f = (v as f64) / (u32::MAX as f64 + 1.0);
        out.push(f);
        i += 4;
    }
    out
}

pub struct ProvablyFairRng {
    pub server_seed: String, // secret
    pub client_seed: String,
    pub nonce: u64,
}

impl ProvablyFairRng {
    pub fn new(server_seed: impl Into<String>, client_seed: impl Into<String>, nonce: u64) -> Self {
        Self {
            server_seed: server_seed.into(),
            client_seed: client_seed.into(),
            nonce,
        }
    }

    pub fn server_seed_hash_hex(&self) -> String {
        derive_hash_hex(self.server_seed.as_bytes())
    }

    pub fn hmac_bytes(&self) -> [u8; 32] {
        let mut mac = HmacSha256::new_from_slice(self.server_seed.as_bytes()).expect("HMAC key");
        let msg = format!("{}:{}", self.client_seed, self.nonce);
        mac.update(msg.as_bytes());
        let res = mac.finalize().into_bytes();
        let mut out = [0u8; 32];
        out.copy_from_slice(&res);
        out
    }

    pub fn next_floats(&self, count: usize) -> Vec<f64> {
        let bytes = self.hmac_bytes();
        derive_floats(&bytes, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism() {
        let rng1 = ProvablyFairRng::new("server", "client", 1);
        let rng2 = ProvablyFairRng::new("server", "client", 1);
        assert_eq!(rng1.server_seed_hash_hex(), rng2.server_seed_hash_hex());
        assert_eq!(rng1.hmac_bytes().to_vec(), rng2.hmac_bytes().to_vec());
        assert_eq!(rng1.next_floats(5), rng2.next_floats(5));
    }
}
