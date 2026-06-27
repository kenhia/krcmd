//! Shared wire protocol, SSHSIG-based request auth, and command definitions
//! used by both `krcmd` (the remote CLI) and `krcmd-host` (the Windows daemon).
//!
//! The two sides only ever agree on *named* commands defined in [`commands`];
//! there is deliberately no facility for arbitrary command execution.

pub mod auth;
pub mod commands;
pub mod protocol;

pub use protocol::{now_unix, random_nonce, Request, Response, Signable, DEFAULT_PORT, NAMESPACE};
