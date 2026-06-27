//! In-memory replay protection: reject a previously seen `(identity, nonce)`.
//!
//! Combined with the timestamp window enforced in `server.rs`, the cache only
//! needs to remember nonces for roughly that window, so it is pruned on access.

use std::collections::HashMap;
use std::sync::Mutex;

pub struct ReplayGuard {
    window_secs: u64,
    seen: Mutex<HashMap<(String, String), u64>>,
}

impl ReplayGuard {
    #[must_use]
    pub fn new(window_secs: u64) -> Self {
        Self {
            window_secs,
            seen: Mutex::new(HashMap::new()),
        }
    }

    /// Returns `true` if the nonce is fresh (and records it); `false` if it was
    /// already seen.
    pub fn check_and_record(&self, identity: &str, nonce: &str) -> bool {
        let now = krcmd_proto::now_unix();
        let mut map = self.seen.lock().expect("replay mutex");
        let keep = self.window_secs.saturating_mul(2);
        map.retain(|_, &mut t| now.saturating_sub(t) <= keep);

        let key = (identity.to_string(), nonce.to_string());
        if map.contains_key(&key) {
            return false;
        }
        map.insert(key, now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_then_replayed() {
        let g = ReplayGuard::new(60);
        assert!(g.check_and_record("ken@kai", "nonce-1"));
        assert!(!g.check_and_record("ken@kai", "nonce-1"));
        assert!(g.check_and_record("ken@kai", "nonce-2"));
        assert!(g.check_and_record("ken@kubs", "nonce-1"));
    }
}
