//! Resolve the effective settings for a request from flags, env, and sensible
//! defaults (SSH keys, `$SSH_CLIENT`, hostname).

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

/// Flags shared by every subcommand. Each also reads an env var.
#[derive(Debug, clap::Args)]
pub struct CommonArgs {
    /// Host running krcmd-host, as `host` or `host:port`.
    /// Defaults to the first field of `$SSH_CLIENT`.
    #[arg(short = 's', long, env = "KRCMD_HOST", global = true)]
    pub server: Option<String>,

    /// Port to use when `--server` has none.
    #[arg(long, env = "KRCMD_PORT", global = true)]
    pub port: Option<u16>,

    /// Signer identity; must match an allowed_signers entry on the host.
    /// Defaults to `<user>@<ssh-host>`.
    #[arg(short = 'i', long, env = "KRCMD_IDENTITY", global = true)]
    pub identity: Option<String>,

    /// Path to the OpenSSH private key used to sign. Defaults to `~/.ssh/id_ed25519`.
    #[arg(short = 'k', long, env = "KRCMD_KEY", global = true)]
    pub key: Option<String>,

    /// SSH host/alias the host should use to reach this box.
    /// Defaults to this machine's hostname.
    #[arg(short = 'H', long = "host", env = "KRCMD_SSH_HOST", global = true)]
    pub ssh_host: Option<String>,

    /// Remote user the host should connect as. Defaults to the current user.
    #[arg(short = 'u', long, env = "KRCMD_USER", global = true)]
    pub user: Option<String>,

    /// Build and sign the request, print it, but do not send.
    #[arg(long, global = true)]
    pub dry_run: bool,
}

/// Fully resolved settings for a single invocation.
#[derive(Debug)]
pub struct Resolved {
    pub endpoint: String,
    pub identity: String,
    pub key_path: PathBuf,
    pub user: String,
    pub ssh_host: String,
    pub dry_run: bool,
}

impl CommonArgs {
    pub fn resolve(&self) -> Result<Resolved> {
        let user = self.user.clone().unwrap_or_else(current_user);
        let ssh_host = self.ssh_host.clone().unwrap_or_else(current_hostname);
        let identity = self
            .identity
            .clone()
            .unwrap_or_else(|| format!("{user}@{ssh_host}"));
        let key_path = self.key.as_deref().map_or_else(default_key, expand_tilde);
        let endpoint = self.resolve_endpoint()?;

        Ok(Resolved {
            endpoint,
            identity,
            key_path,
            user,
            ssh_host,
            dry_run: self.dry_run,
        })
    }

    fn resolve_endpoint(&self) -> Result<String> {
        let host_port = match &self.server {
            Some(s) => s.clone(),
            None => host_from_ssh_client().ok_or_else(|| {
                anyhow!("no --server/$KRCMD_HOST and $SSH_CLIENT is unset; cannot find host")
            })?,
        };
        let host_port = if host_port.contains(':') {
            host_port
        } else {
            let port = self.port.unwrap_or(krcmd_proto::DEFAULT_PORT);
            format!("{host_port}:{port}")
        };
        Ok(format!("http://{host_port}/command"))
    }
}

fn host_from_ssh_client() -> Option<String> {
    let v = std::env::var("SSH_CLIENT").ok()?;
    v.split_whitespace().next().map(ToString::to_string)
}

fn current_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| whoami::username())
}

fn current_hostname() -> String {
    whoami::fallible::hostname().unwrap_or_else(|_| "localhost".to_string())
}

fn default_key() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".ssh")
        .join("id_ed25519")
}

fn expand_tilde(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/").or_else(|| s.strip_prefix("~\\")) {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(s)
}

/// Turn a user-supplied path (possibly `.` or relative) into an absolute string.
pub fn resolve_path(input: &str) -> Result<String> {
    let cwd = std::env::current_dir()?;
    let p = if input == "." {
        cwd
    } else {
        let pp = Path::new(input);
        if pp.is_absolute() {
            pp.to_path_buf()
        } else {
            cwd.join(pp)
        }
    };
    let p = p.canonicalize().unwrap_or(p);
    let mut s = p.to_string_lossy().into_owned();
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        s = rest.to_string();
    }
    Ok(s)
}
