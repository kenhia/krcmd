//! The fixed set of commands understood by the protocol.
//!
//! Each command has a stable name and a typed argument struct shared by both
//! ends. The host only executes commands it has a registered handler for, and
//! a handler only ever runs with these validated, typed arguments — there is no
//! path for ad-hoc/arbitrary execution.

use serde::{Deserialize, Serialize};

/// Launch (remote) VS Code on the host. Command name on the wire.
pub const LAUNCH_CODE: &str = "launch-code";

/// Which VS Code flavor to launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeVariant {
    Stable,
    Insiders,
}

impl CodeVariant {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            CodeVariant::Stable => "stable",
            CodeVariant::Insiders => "insiders",
        }
    }
}

/// Arguments for [`LAUNCH_CODE`].
///
/// The host builds:
/// `<code|code-insiders> --folder-uri vscode-remote://ssh-remote+{user}@{ssh_host}{path}`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchCodeArgs {
    pub variant: CodeVariant,
    /// Remote user the host should SSH back as, e.g. `ken`.
    pub user: String,
    /// SSH host/alias the host uses to reach the remote, e.g. `kai`.
    pub ssh_host: String,
    /// Absolute path on the remote to open.
    pub path: String,
}

impl LaunchCodeArgs {
    /// The `--folder-uri` value the host will pass to the VS Code binary.
    #[must_use]
    pub fn folder_uri(&self) -> String {
        format!(
            "vscode-remote://ssh-remote+{}@{}{}",
            self.user, self.ssh_host, self.path
        )
    }
}
