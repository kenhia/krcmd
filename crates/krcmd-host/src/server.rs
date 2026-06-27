//! HTTP surface and the verify-then-dispatch request pipeline.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use colored::Colorize;
use krcmd_proto::auth::AllowedSigners;
use krcmd_proto::{now_unix, Request, Response};

use crate::config::Config;
use crate::registry::{CmdError, HostCtx, Registry};
use crate::replay::ReplayGuard;

pub struct AppState {
    pub config: Config,
    pub signers: AllowedSigners,
    pub registry: Registry,
    pub replay: ReplayGuard,
    pub dry_run: bool,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/command", post(handle_command))
        .route("/health", get(|| async { "ok" }))
        .with_state(state)
}

async fn handle_command(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Request>,
) -> (StatusCode, Json<Response>) {
    match process(&state, req) {
        Ok(resp) => {
            println!("{} {}", "[ok]".green().bold(), resp.message);
            (StatusCode::OK, Json(resp))
        }
        Err((code, msg)) => {
            println!("{} {}", "[denied]".red().bold(), msg);
            (code, Json(Response::error(msg)))
        }
    }
}

/// The pipeline: timestamp window -> signature -> replay -> dispatch.
/// Signature is verified before the nonce is recorded so invalid requests
/// cannot poison the replay cache.
fn process(state: &AppState, req: Request) -> Result<Response, (StatusCode, String)> {
    let signable = req.signable();

    let now = now_unix();
    let skew = state.config.max_skew_secs;
    if now.abs_diff(signable.ts) > skew {
        return Err((
            StatusCode::UNAUTHORIZED,
            format!(
                "stale timestamp (now={now}, ts={}, max_skew={skew}s)",
                signable.ts
            ),
        ));
    }

    state
        .signers
        .verify_request(&signable, &req.signature)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    if !state
        .replay
        .check_and_record(&signable.identity, &signable.nonce)
    {
        return Err((StatusCode::UNAUTHORIZED, "replayed nonce".into()));
    }

    let ctx = HostCtx {
        config: &state.config,
        dry_run: state.dry_run,
    };
    state
        .registry
        .dispatch(&signable.command, req.payload, &ctx)
        .map_err(|e| {
            let code = match e {
                CmdError::Unknown(_) => StatusCode::NOT_FOUND,
                CmdError::BadArgs(_) => StatusCode::BAD_REQUEST,
                CmdError::NotConfigured(_) => StatusCode::SERVICE_UNAVAILABLE,
                CmdError::Exec(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (code, e.to_string())
        })
}
