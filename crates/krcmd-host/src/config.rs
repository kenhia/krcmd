//! Host configuration: a TOML file plus environment-variable overrides.
//!
//! Search order for the file:
//! 1. `$KRCMD_HOST_CONFIG`
//! 2. `./krcmd-host.toml`
//! 3. `<config-dir>/krcmd/krcmd-host.toml`
//!
//! Every field can also be supplied/overridden via env, so the daemon can run
//! with no file at all if the env provides what's needed.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Socket address to bind, e.g. `0.0.0.0:42271`.
    pub bind: String,
    /// Path to the trust list of allowed signer public keys.
    pub allowed_signers: Option<PathBuf>,
    /// Accepted clock skew (seconds) for request timestamps.
    pub max_skew_secs: u64,
    pub commands: Commands,
    /// The config file this was loaded from, if any (not part of the file).
    #[serde(skip)]
    pub source: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind: format!("0.0.0.0:{}", krcmd_proto::DEFAULT_PORT),
            allowed_signers: None,
            max_skew_secs: 60,
            commands: Commands::default(),
            source: None,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Commands {
    #[serde(rename = "launch-code")]
    pub launch_code: LaunchCodeConfig,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct LaunchCodeConfig {
    /// Path to the stable VS Code launcher (e.g. `code.cmd`).
    pub stable_path: Option<String>,
    /// Path to the VS Code Insiders launcher (e.g. `code-insiders.cmd`).
    pub insiders_path: Option<String>,
}

impl Config {
    /// Load config from the first file found, then apply env overrides.
    pub fn load() -> anyhow::Result<Self> {
        let found = Self::find_path();
        let mut cfg = match &found {
            Some(path) => {
                let text = std::fs::read_to_string(path)
                    .with_context(|| format!("reading {}", path.display()))?;
                toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?
            }
            None => Self::default(),
        };
        cfg.source = found;
        cfg.apply_env_overrides();
        Ok(cfg)
    }

    fn find_path() -> Option<PathBuf> {
        if let Ok(p) = std::env::var("KRCMD_HOST_CONFIG") {
            return Some(PathBuf::from(p));
        }
        // Next to the current working directory.
        let local = PathBuf::from("krcmd-host.toml");
        if local.exists() {
            return Some(local);
        }
        // Next to the executable itself — the natural spot when you drop the
        // binary and its config side by side (e.g. C:\tools\bin\).
        if let Some(p) = exe_dir().map(|d| d.join("krcmd-host.toml")) {
            if p.exists() {
                return Some(p);
            }
        }
        // Per-user default: ~/.config/krcmd-host.toml
        if let Some(p) = config_default_path() {
            if p.exists() {
                return Some(p);
            }
        }
        None
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("KRCMD_BIND") {
            self.bind = v;
        }
        if let Ok(v) = std::env::var("KRCMD_ALLOWED_SIGNERS") {
            self.allowed_signers = Some(PathBuf::from(v));
        }
        if let Ok(v) = std::env::var("KRCMD_MAX_SKEW_SECS") {
            if let Ok(n) = v.parse() {
                self.max_skew_secs = n;
            }
        }
        if let Ok(v) = std::env::var("KRCMD_CODE_PATH") {
            self.commands.launch_code.stable_path = Some(v);
        }
        if let Ok(v) = std::env::var("KRCMD_CODE_INSIDERS_PATH") {
            self.commands.launch_code.insiders_path = Some(v);
        }
    }

    /// Resolve the allowed_signers path (expanding a leading `~`), erroring if unset.
    pub fn allowed_signers_path(&self) -> anyhow::Result<PathBuf> {
        let raw = self.allowed_signers.as_ref().ok_or_else(|| {
            anyhow!("allowed_signers is not set (config file or KRCMD_ALLOWED_SIGNERS)")
        })?;
        Ok(expand_tilde(raw))
    }
}

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(Path::to_path_buf)
}

/// The per-user default config location: `~/.config/krcmd-host.toml`
/// (on Windows, `~` is the user profile directory).
fn config_default_path() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(".config").join("krcmd-host.toml"))
}

fn expand_tilde(path: &Path) -> PathBuf {
    if let Ok(rest) = path.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilde_expands_to_home() {
        let home = dirs::home_dir().expect("home dir");
        // Forward slashes work cross-platform; strip_prefix splits on components.
        assert_eq!(
            expand_tilde(Path::new("~/.ssh/allowed_signers")),
            home.join(".ssh").join("allowed_signers")
        );
    }

    #[test]
    fn absolute_path_is_unchanged() {
        let p = if cfg!(windows) {
            r"C:\Users\kenhi\.ssh\allowed_signers"
        } else {
            "/home/ken/.ssh/allowed_signers"
        };
        assert_eq!(expand_tilde(Path::new(p)), PathBuf::from(p));
    }
}
