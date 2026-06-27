//! krcmd-host — the daemon that runs on the Windows host. It verifies SSHSIG-
//! signed requests against a trust list and dispatches them to registered
//! command handlers. There is no facility for arbitrary command execution.

mod commands;
mod config;
mod registry;
mod replay;
mod server;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use colored::Colorize;
use krcmd_proto::auth::AllowedSigners;

use crate::config::Config;
use crate::registry::Registry;
use crate::replay::ReplayGuard;
use crate::server::AppState;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{} {e:#}", "error:".red().bold());
        std::process::exit(1);
    }
}

/// Build the set of commands the host is willing to run. Add new commands here.
fn build_registry() -> Registry {
    let mut r = Registry::new();
    r.register(commands::launch_code::LaunchCode);
    r
}

async fn run() -> anyhow::Result<()> {
    let dry_run =
        std::env::args().any(|a| a == "--dry-run") || std::env::var("KRCMD_DRY_RUN").is_ok();

    let config = Config::load()?;
    let bind: SocketAddr = config
        .bind
        .parse()
        .with_context(|| format!("invalid bind address: {}", config.bind))?;
    let signers_path = config.allowed_signers_path()?;
    let signers = AllowedSigners::load(&signers_path)
        .with_context(|| format!("loading allowed_signers from {}", signers_path.display()))?;
    if signers.is_empty() {
        anyhow::bail!("allowed_signers is empty: {}", signers_path.display());
    }

    let registry = build_registry();
    let skew = config.max_skew_secs;

    println!("{}", "krcmd-host".cyan().bold());
    println!("  bind            {bind}");
    println!(
        "  allowed signers {} ({})",
        signers.len(),
        signers_path.display()
    );
    println!("  commands        {}", registry.names().join(", "));
    println!("  max skew        {skew}s");
    if dry_run {
        println!("  {}", "DRY RUN (commands will not execute)".yellow());
    }

    let state = Arc::new(AppState {
        config,
        signers,
        registry,
        replay: ReplayGuard::new(skew),
        dry_run,
    });

    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .with_context(|| format!("binding {bind}"))?;
    println!("{} listening on http://{bind}", "ready:".green().bold());

    axum::serve(listener, server::router(state))
        .await
        .context("server error")?;
    Ok(())
}
