//! The command framework: a registry of named [`Handler`]s.
//!
//! The host can only run commands that have a registered handler, and a handler
//! only ever sees typed, validated arguments. Adding a command means adding a
//! handler and registering it in `build_registry` (see `main.rs`) — there is no
//! mechanism for ad-hoc execution.

use std::collections::HashMap;

use krcmd_proto::Response;
use serde_json::Value;

use crate::config::Config;

/// Per-request context handed to each handler.
pub struct HostCtx<'a> {
    pub config: &'a Config,
    /// When true, handlers describe what they would do without executing.
    pub dry_run: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CmdError {
    #[error("unknown command: {0}")]
    Unknown(String),
    #[error("invalid arguments: {0}")]
    BadArgs(String),
    #[error("command not configured: {0}")]
    NotConfigured(String),
    #[error("execution failed: {0}")]
    Exec(String),
}

/// A single named command the host knows how to run.
pub trait Handler: Send + Sync {
    fn name(&self) -> &'static str;
    fn run(&self, payload: Value, ctx: &HostCtx) -> Result<Response, CmdError>;
}

#[derive(Default)]
pub struct Registry {
    handlers: HashMap<&'static str, Box<dyn Handler>>,
}

impl Registry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handler: impl Handler + 'static) -> &mut Self {
        self.handlers.insert(handler.name(), Box::new(handler));
        self
    }

    pub fn dispatch(
        &self,
        command: &str,
        payload: Value,
        ctx: &HostCtx,
    ) -> Result<Response, CmdError> {
        let handler = self
            .handlers
            .get(command)
            .ok_or_else(|| CmdError::Unknown(command.to_string()))?;
        handler.run(payload, ctx)
    }

    /// Sorted list of registered command names (for startup logging).
    #[must_use]
    pub fn names(&self) -> Vec<&'static str> {
        let mut v: Vec<&'static str> = self.handlers.keys().copied().collect();
        v.sort_unstable();
        v
    }
}
