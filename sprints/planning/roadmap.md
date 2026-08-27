# Roadmap

> The general plan for this project. Keep it current; detail lives in the
> sprint records.

## Now

(nothing in flight)

## Done

- **003 — CI + krcmd-host autostart.** `ci.yml` (kpolice's shape),
  `rust-toolchain.toml` pin (1.98.0), logon scheduled task on cleo.
- **002 — store publish + fleet deploy.** Commit-stamped `--version`,
  `just publish` (Linux client → package store), `just deploy` (knarr →
  kai, kubs0). First consumer of the build-clones skill. korg program 1675.
- **001 — onto the kprojects harness.** Layout, `just check`, agent
  instruction files. No behaviour change.

## Later / Ideas

- More commands. The framework is built for it — a command is one args struct,
  one `Handler`, one subcommand — but `launch-code` is still the only one.
- `krcmd-host.exe` via the package store, once a Windows install story
  exists (kpolice `install-cleo.ps1` precedent; would add the
  `x86_64-pc-windows-gnu` target back to the toolchain pin).
