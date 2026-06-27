//! Build, sign, and send a request to the host daemon.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use krcmd_proto::{auth, now_unix, random_nonce, Request, Response, Signable};
use serde_json::Value;
use ssh_key::PrivateKey;

use crate::config::Resolved;

/// Sign and send `command`/`payload`. `preview` is a human-readable summary of
/// what the host will do (shown for `--dry-run`).
pub fn send(cfg: &Resolved, command: &str, payload: Value, preview: &str) -> Result<()> {
    let key = read_key(&cfg.key_path)?;

    let signable = Signable {
        identity: cfg.identity.clone(),
        command: command.to_string(),
        payload: payload.clone(),
        ts: now_unix(),
        nonce: random_nonce(),
    };
    let signature = auth::sign(&key, &signable).context("signing request")?;

    let request = Request {
        identity: signable.identity,
        command: signable.command,
        payload,
        ts: signable.ts,
        nonce: signable.nonce,
        signature,
    };

    if cfg.dry_run {
        println!("dry-run (not sent)");
        println!("  endpoint  {}", cfg.endpoint);
        println!("  identity  {}", request.identity);
        println!("  command   {command}");
        println!("  would run {preview}");
        println!("{}", serde_json::to_string_pretty(&request)?);
        return Ok(());
    }

    match ureq::post(&cfg.endpoint).send_json(&request) {
        Ok(r) => {
            report(&r.into_json::<Response>().context("decoding response")?);
            Ok(())
        }
        Err(ureq::Error::Status(code, r)) => {
            let body = r
                .into_json::<Response>()
                .unwrap_or_else(|_| Response::error(format!("HTTP {code}")));
            report(&body);
            Err(anyhow!("request denied (HTTP {code})"))
        }
        Err(e) => Err(anyhow!("network error contacting {}: {e}", cfg.endpoint)),
    }
}

fn read_key(path: &Path) -> Result<PrivateKey> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading private key {}", path.display()))?;
    let key = PrivateKey::from_openssh(&text)
        .with_context(|| format!("parsing private key {}", path.display()))?;
    if key.is_encrypted() {
        return Err(anyhow!(
            "private key {} is passphrase-protected; krcmd needs an unencrypted ed25519 key \
             (or point --key at one)",
            path.display()
        ));
    }
    Ok(key)
}

fn report(body: &Response) {
    let tag = if body.ok { "ok" } else { "denied" };
    println!("[{tag}] {}", body.message);
    if let Some(detail) = &body.detail {
        println!("       {detail}");
    }
}
