//! Embed the git commit so `krcmd --version` can tell one build from another.
//!
//! Not cosmetic: `just publish` re-reads the built binary with `--version` and
//! publishes it under the label that stamp produces, so the stamp and the
//! store label are one fact rather than two that can drift. A `dirty` or
//! `unknown` stamp is refused at publish time — a published version must name
//! a commit someone can check out.
//!
//! Mirrors kpolice's `build.rs` (itself a deliberate divergence from kaed's:
//! the second `--version` field is the store label VERBATIM, e.g.
//! `0.1.0-ae575ac`, because knarr's confirm step tests whether `--version`
//! output CONTAINS the label it deployed). One adaptation: krcmd is a
//! workspace, so this build script's cwd is `crates/krcmd/`, not the repo
//! root — the `.git` paths for rerun-if-changed are resolved through git
//! rather than assumed relative.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Rerun when HEAD moves or the index changes; otherwise cargo caches the
    // stamp from whichever commit happened to be checked out first.
    if let Some(git_dir) = read_git("rev-parse --git-dir").map(PathBuf::from) {
        for p in ["HEAD", "index"] {
            let path = git_dir.join(p);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
        // `.git/HEAD` on a branch points at a ref whose file moves on commit.
        if let Some(head) = read_git("rev-parse --symbolic-full-name HEAD") {
            let refpath = git_dir.join(&head);
            if refpath.exists() {
                println!("cargo:rerun-if-changed={}", refpath.display());
            }
        }
    }

    let describe = read_git("describe --always --dirty").unwrap_or_else(|| "unknown".into());
    let date = read_git("log -1 --format=%cd --date=short").unwrap_or_else(|| "unknown".into());
    let crate_version = std::env::var("CARGO_PKG_VERSION").expect("cargo sets this");

    // Composed here rather than in the crate so it can be a `&'static str`:
    // clap's `version` takes one, and a `String` would have to be leaked.
    let full = if describe == "unknown" {
        format!("{crate_version}-unknown")
    } else {
        format!("{crate_version}-{describe} ({date})")
    };

    println!("cargo:rustc-env=KRCMD_VERSION_FULL={full}");
}

/// Run `git <args>` and return trimmed stdout, or `None` if git is missing,
/// this is not a repository, or the command failed for any other reason.
fn read_git(args: &str) -> Option<String> {
    let out = Command::new("git")
        .args(args.split_whitespace())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
