//! Wire types and the canonical bytes that requests are signed over.

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// SSHSIG namespace; binds a signature to this application.
pub const NAMESPACE: &str = "krcmd";

/// Default TCP port the host daemon listens on.
pub const DEFAULT_PORT: u16 = 42271;

/// The portion of a [`Request`] that is covered by the signature.
///
/// Both ends build an identical `Signable` and run [`Signable::signing_bytes`]
/// to obtain the exact bytes that are signed/verified. `serde_json` emits
/// struct fields in declaration order, and (with default features) backs JSON
/// objects with a `BTreeMap`, so `payload` keys are serialized in sorted order
/// — the encoding is therefore deterministic across both sides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signable {
    pub identity: String,
    pub command: String,
    pub payload: Value,
    pub ts: u64,
    pub nonce: String,
}

impl Signable {
    /// Deterministic bytes that both sides sign and verify.
    pub fn signing_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Signable always serializes")
    }
}

/// A full request as sent over the wire (`POST /command`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub identity: String,
    pub command: String,
    pub payload: Value,
    pub ts: u64,
    pub nonce: String,
    /// PEM-armored SSHSIG signature over [`Signable::signing_bytes`].
    pub signature: String,
}

impl Request {
    /// Reconstruct the signed portion of this request for verification.
    pub fn signable(&self) -> Signable {
        Signable {
            identity: self.identity.clone(),
            command: self.command.clone(),
            payload: self.payload.clone(),
            ts: self.ts,
            nonce: self.nonce.clone(),
        }
    }
}

/// The host's reply to a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub ok: bool,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl Response {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
            detail: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
            detail: None,
        }
    }

    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Current Unix time in seconds.
pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// A fresh base64 nonce (128 bits) for replay protection.
pub fn random_nonce() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
