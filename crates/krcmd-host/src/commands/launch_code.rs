//! `launch-code`: open (remote) VS Code on the host machine.

use std::process::Command;

use krcmd_proto::commands::{CodeVariant, LaunchCodeArgs, LAUNCH_CODE};
use krcmd_proto::Response;
use serde_json::Value;

use crate::registry::{CmdError, Handler, HostCtx};

pub struct LaunchCode;

impl Handler for LaunchCode {
    fn name(&self) -> &'static str {
        LAUNCH_CODE
    }

    fn run(&self, payload: Value, ctx: &HostCtx) -> Result<Response, CmdError> {
        let args: LaunchCodeArgs =
            serde_json::from_value(payload).map_err(|e| CmdError::BadArgs(e.to_string()))?;
        validate(&args)?;

        let cfg = &ctx.config.commands.launch_code;
        let exe = match args.variant {
            CodeVariant::Stable => cfg.stable_path.clone(),
            CodeVariant::Insiders => cfg.insiders_path.clone(),
        }
        .ok_or_else(|| {
            CmdError::NotConfigured(format!("{LAUNCH_CODE}.{}_path", args.variant.as_str()))
        })?;

        let uri = args.folder_uri();
        let pretty = format!("{exe} --folder-uri {uri}");

        if ctx.dry_run {
            return Ok(Response::ok(format!(
                "[dry-run] would launch {} VS Code",
                args.variant.as_str()
            ))
            .with_detail(pretty));
        }

        match launch_command(&exe, &uri).spawn() {
            Ok(_child) => Ok(Response::ok(format!(
                "launched {} VS Code at {}@{}{}",
                args.variant.as_str(),
                args.user,
                args.ssh_host,
                args.path
            ))
            .with_detail(uri)),
            Err(e) => Err(CmdError::Exec(format!("failed to start {exe}: {e}"))),
        }
    }
}

/// Build the OS command. On Windows, `.cmd`/`.bat` launchers must run through
/// `cmd /C`; real executables are spawned directly.
fn launch_command(exe: &str, uri: &str) -> Command {
    let lower = exe.to_ascii_lowercase();
    if cfg!(windows) && (lower.ends_with(".cmd") || lower.ends_with(".bat")) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(exe).arg("--folder-uri").arg(uri);
        c
    } else {
        let mut c = Command::new(exe);
        c.arg("--folder-uri").arg(uri);
        c
    }
}

/// Reject anything that isn't a plain identity token or an absolute path.
/// Arguments are passed to `Command` directly (no shell), but we still validate
/// defensively.
fn validate(args: &LaunchCodeArgs) -> Result<(), CmdError> {
    if !is_token(&args.user) {
        return Err(CmdError::BadArgs(format!("invalid user: {:?}", args.user)));
    }
    if !is_token(&args.ssh_host) {
        return Err(CmdError::BadArgs(format!(
            "invalid ssh_host: {:?}",
            args.ssh_host
        )));
    }
    let p = &args.path;
    if p.is_empty() || p.contains(['\n', '\r', '"', '\0']) {
        return Err(CmdError::BadArgs("path contains illegal characters".into()));
    }
    let absolute = p.starts_with('/') || is_windows_abs(p);
    if !absolute {
        return Err(CmdError::BadArgs(format!("path must be absolute: {p:?}")));
    }
    Ok(())
}

fn is_token(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@'))
}

fn is_windows_abs(p: &str) -> bool {
    let b = p.as_bytes();
    b.len() > 2 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(path: &str) -> LaunchCodeArgs {
        LaunchCodeArgs {
            variant: CodeVariant::Insiders,
            user: "ken".into(),
            ssh_host: "kai".into(),
            path: path.into(),
        }
    }

    #[test]
    fn accepts_absolute_unix_path() {
        assert!(validate(&args("/home/ken/src/tools/widget")).is_ok());
    }

    #[test]
    fn rejects_relative_path() {
        assert!(validate(&args("src/tools/widget")).is_err());
    }

    #[test]
    fn rejects_path_with_quote() {
        assert!(validate(&args("/home/\"; rm -rf x")).is_err());
    }

    #[test]
    fn rejects_bad_user() {
        let mut a = args("/home/ken");
        a.user = "ken; whoami".into();
        assert!(validate(&a).is_err());
    }

    #[test]
    fn folder_uri_is_well_formed() {
        let a = args("/home/ken/x");
        assert_eq!(
            a.folder_uri(),
            "vscode-remote://ssh-remote+ken@kai/home/ken/x"
        );
    }
}
