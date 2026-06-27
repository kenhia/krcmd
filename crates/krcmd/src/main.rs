//! krcmd — the remote-side CLI. Run on a dev box you're SSH'd into; it signs a
//! named command with your SSH key and sends it to the host daemon.

mod client;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use krcmd_proto::commands::{CodeVariant, LaunchCodeArgs, LAUNCH_CODE};

use crate::config::{resolve_path, CommonArgs};

#[derive(Parser)]
#[command(
    name = "krcmd",
    version,
    about = "Send signed commands to a krcmd host"
)]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Launch VS Code (stable) on the host, connected back to this box.
    Vsc {
        /// Path to open (default: current directory).
        #[arg(default_value = ".")]
        path: String,
    },
    /// Launch VS Code Insiders on the host, connected back to this box.
    Vsci {
        /// Path to open (default: current directory).
        #[arg(default_value = ".")]
        path: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let cfg = cli.common.resolve()?;

    let (variant, path) = match &cli.command {
        Cmd::Vsc { path } => (CodeVariant::Stable, path),
        Cmd::Vsci { path } => (CodeVariant::Insiders, path),
    };

    let args = LaunchCodeArgs {
        variant,
        user: cfg.user.clone(),
        ssh_host: cfg.ssh_host.clone(),
        path: resolve_path(path)?,
    };
    let preview = args.folder_uri();
    let payload = serde_json::to_value(&args)?;

    client::send(&cfg, LAUNCH_CODE, payload, &preview)
}
