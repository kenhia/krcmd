# 002 — store publish, and the client to kai + kubs0

korg: proposal 1674 (WIs 1670, 1671, 1672), slice 2 of program 1675
("krcmd everywhere via build-clones"). First real consumer of the
build-clones skill (agent-skills sprint 1669).

## Goal

Get the `krcmd` client onto kubs0 (and re-onto kai) the store-native way:
publish recipe → kpkg store → knarr deploy, with exactly one copy on PATH
per host. cleo stays the canonical (and only) clone; Linux builds happen in
the kai clone cache via the build-clones skill.

## What shipped in the repo (WI 1670)

- `crates/krcmd/build.rs` — commit-stamps the client so `krcmd --version`
  prints the store label verbatim (`0.1.0-<sha> (<date>)`, second field is
  the label). Mirrors kpolice's build.rs, with one adaptation: krcmd is a
  workspace, so the `.git` rerun-if-changed paths are resolved via
  `git rev-parse --git-dir` instead of assumed relative to the crate.
- `justfile` gains `version`, `publish`, `deploy`:
  - `publish` — Linux client only, deliberately; `krcmd-host.exe`
    store-distribution waits on a Windows install story (kpolice
    `install-cleo.ps1` precedent). Refuses dirty trees and
    `dirty`/`unknown` stamps; `kpkg --no-latest` off main.
  - `deploy` — `knarr deploy krcmd --host kai,kubs0` with `--store`
    baked in (`KNARR_STORE_URL` is set nowhere on the fleet — recorded
    gotcha). knarr defaults supply `--dest /usr/local/bin/krcmd` and
    `--file krcmd-x86_64-linux`.

## Ops (WIs 1671, 1672) — recorded here as they land

- Publish + deploy via build-clones: see korg WI 1671 comments.
- kai cleanup: remove the hand-copied `~/.local/bin/krcmd` (June 26 build)
  so `which -a krcmd` shows exactly one hit.
- cleo trust list: `ken@kubs0` line added to `~/.ssh/allowed_signers`,
  daemon restarted. How the daemon is launched on cleo is documented in
  korg WI 1672 as part of this sprint.

## Decisions

- Dest is `/usr/local/bin/krcmd` (knarr's proven groove), not
  `~/.local/bin`: Ken's requirement is "on PATH", location flexible — so
  take the path knarr already exercises.
- Client artifact only, one store version. When the host exe joins, it
  joins the same version (never a per-platform version split).
