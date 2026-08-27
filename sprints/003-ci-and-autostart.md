# 003 — CI, and a krcmd-host autostart on cleo

korg: proposal 1678 (WIs 1285, 1677). Follow-up hardening after program 1675.

## CI (WI 1285)

`.github/workflows/ci.yml` mirrors kpolice's (which mirrors korg's — the
mature shape in the family, per kpolice WI-1509's routing): `just check`
inlined step-by-step, branch-gated via PR / main gated on the merge commit,
concurrency that cancels superseded runs on branches but never on main.

That workflow shape wants the toolchain pinned in `rust-toolchain.toml`
(CI runs `rustup toolchain install` with no arguments and reads the file),
so this sprint also lands the roadmap's "Later" item: **1.98.0** pinned
(matching kai and kpolice; korg WI-1564 owns moving it), `rustfmt` +
`clippy` components, no extra targets — publish ships the Linux client
only. `rustfmt.toml` mirrored too (edition 2021 here).

Pre-ship verification per the run-real-CI-locally rule, both builders:

- cleo (Windows): gate green on freshly auto-installed 1.98.0 — fmt OK,
  clippy clean across the workspace, 14 tests pass.
- kai (Linux, the CI-like environment): whole-workspace gate run in the
  build-clones cache against this branch — the first time `krcmd-host` was
  ever compiled on Linux, which is exactly what CI will do on every push.

## Autostart (WI 1677)

Sprint 002's close-out found krcmd-host had **no autostart** and was not
running — `krcmd` from kai had been silently dead-at-rest. Now:
`scripts/install-autostart-cleo.ps1` registers Scheduled Task `krcmd-host`
— at kenhi's logon, interactive session (the daemon launches VS Code onto
the desktop; a service would be wrong), hidden PowerShell wrapper that
`Start-Process`-es the daemon detached with stdout/stderr to
`~/.config/krcmd-host.log`/`.err.log`, then exits. Idempotent; defaults
disarmed: 5-min ExecutionTimeLimit (the wrapper, not the daemon), battery
rules off, IgnoreNew for multiple instances.

Registered on cleo and verified live: existing daemon stopped,
`Start-ScheduledTask`, port 42271 listening, banner counts 2 signers, and
a real `krcmd vsci .` from kubs0 landed through the task-started instance.

`docs/host-cleo.md` rewritten for the new reality (manual-launch era kept
as history).
