//! SSHSIG-based signing and verification, reusing OpenSSH ed25519 keys.
//!
//! The remote signs a request's [`Signable`] bytes with its private key; the
//! host verifies against a trust list of public keys ([`AllowedSigners`]) keyed
//! by identity. This is the same key material and trust model as SSH itself.

use std::collections::HashMap;
use std::path::Path;

use ssh_key::{HashAlg, LineEnding, PrivateKey, PublicKey, SshSig};

use crate::protocol::{Signable, NAMESPACE};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("ssh key error: {0}")]
    Ssh(#[from] ssh_key::Error),
    #[error("unknown identity: {0}")]
    UnknownIdentity(String),
    #[error("signature verification failed")]
    BadSignature,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("malformed allowed_signers entry on line {0}")]
    BadSignerLine(usize),
}

/// Sign a request's canonical bytes, returning a PEM-armored SSHSIG.
///
/// `key` must be an unencrypted OpenSSH private key (ed25519).
pub fn sign(key: &PrivateKey, signable: &Signable) -> Result<String, AuthError> {
    let sig = SshSig::sign(key, NAMESPACE, HashAlg::Sha512, &signable.signing_bytes())?;
    Ok(sig.to_pem(LineEnding::LF)?)
}

/// Verify a PEM-armored SSHSIG against a known public key.
pub fn verify(
    pubkey: &PublicKey,
    signable: &Signable,
    signature_pem: &str,
) -> Result<(), AuthError> {
    let sig: SshSig = signature_pem.parse()?;
    pubkey
        .verify(NAMESPACE, &signable.signing_bytes(), &sig)
        .map_err(|_| AuthError::BadSignature)
}

/// A trust list mapping signer identity -> public key.
///
/// File format (one entry per line; `#` comments and blank lines ignored):
///
/// ```text
/// ken@kai    ssh-ed25519 AAAAC3Nza... optional-comment
/// ken@kubs   ssh-ed25519 AAAAC3Nza... optional-comment
/// ```
///
/// The first whitespace-delimited token is the identity; the remainder is an
/// OpenSSH public key line. Generate entries from each box's
/// `~/.ssh/id_ed25519.pub`.
#[derive(Debug, Default, Clone)]
pub struct AllowedSigners {
    keys: HashMap<String, PublicKey>,
}

impl AllowedSigners {
    /// Parse trust-list contents.
    pub fn parse(contents: &str) -> Result<Self, AuthError> {
        let mut keys = HashMap::new();
        for (idx, raw) in contents.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let lineno = idx + 1;
            let (identity, rest) = line
                .split_once(char::is_whitespace)
                .ok_or(AuthError::BadSignerLine(lineno))?;
            let pubkey = PublicKey::from_openssh(rest.trim())
                .map_err(|_| AuthError::BadSignerLine(lineno))?;
            keys.insert(identity.to_string(), pubkey);
        }
        Ok(Self { keys })
    }

    /// Load and parse a trust-list file.
    pub fn load(path: &Path) -> Result<Self, AuthError> {
        let contents = std::fs::read_to_string(path)?;
        Self::parse(&contents)
    }

    #[must_use]
    pub fn get(&self, identity: &str) -> Option<&PublicKey> {
        self.keys.get(identity)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Look up the request's identity and verify its signature.
    pub fn verify_request(
        &self,
        signable: &Signable,
        signature_pem: &str,
    ) -> Result<(), AuthError> {
        let pubkey = self
            .get(&signable.identity)
            .ok_or_else(|| AuthError::UnknownIdentity(signable.identity.clone()))?;
        verify(pubkey, signable, signature_pem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use ssh_key::Algorithm;

    fn make_key() -> PrivateKey {
        PrivateKey::random(&mut rand::thread_rng(), Algorithm::Ed25519).expect("gen key")
    }

    fn signable(identity: &str) -> Signable {
        Signable {
            identity: identity.to_string(),
            command: "launch-code".into(),
            payload: json!({ "variant": "insiders", "path": "/home/ken/x" }),
            ts: 1_719_400_000,
            nonce: "abcd1234".into(),
        }
    }

    fn signers_file(identity: &str, key: &PrivateKey) -> String {
        let publine = key.public_key().to_openssh().expect("openssh pub");
        format!("{identity} {publine}\n")
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let key = make_key();
        let s = signable("ken@kai");
        let sig = sign(&key, &s).unwrap();
        verify(key.public_key(), &s, &sig).unwrap();
    }

    #[test]
    fn tampered_payload_is_rejected() {
        let key = make_key();
        let s = signable("ken@kai");
        let sig = sign(&key, &s).unwrap();

        let mut tampered = s.clone();
        tampered.payload = json!({ "variant": "insiders", "path": "/etc/passwd" });
        let err = verify(key.public_key(), &tampered, &sig).unwrap_err();
        assert!(matches!(err, AuthError::BadSignature));
    }

    #[test]
    fn wrong_key_is_rejected() {
        let signer = make_key();
        let other = make_key();
        let s = signable("ken@kai");
        let sig = sign(&signer, &s).unwrap();
        let err = verify(other.public_key(), &s, &sig).unwrap_err();
        assert!(matches!(err, AuthError::BadSignature));
    }

    #[test]
    fn allowed_signers_parse_and_verify() {
        let key = make_key();
        let file = signers_file("ken@kai", &key);
        let signers = AllowedSigners::parse(&file).unwrap();
        assert_eq!(signers.len(), 1);

        let s = signable("ken@kai");
        let sig = sign(&key, &s).unwrap();
        signers.verify_request(&s, &sig).unwrap();
    }

    #[test]
    fn unknown_identity_is_rejected() {
        let key = make_key();
        let signers = AllowedSigners::parse(&signers_file("ken@kai", &key)).unwrap();
        let s = signable("ken@elsewhere");
        let sig = sign(&key, &s).unwrap();
        let err = signers.verify_request(&s, &sig).unwrap_err();
        assert!(matches!(err, AuthError::UnknownIdentity(_)));
    }

    #[test]
    fn comments_and_blanks_are_ignored() {
        let key = make_key();
        let body = format!(
            "# trust list\n\n   # indented comment\n{}",
            signers_file("ken@kai", &key)
        );
        let signers = AllowedSigners::parse(&body).unwrap();
        assert_eq!(signers.len(), 1);
        assert!(signers.get("ken@kai").is_some());
    }
}
