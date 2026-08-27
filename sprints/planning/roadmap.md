# Roadmap

> The general plan for this project. Keep it current; detail lives in the
> sprint records.

## Now

- **002 — store publish + fleet deploy.** Commit-stamped `--version`,
  `just publish` (Linux client → package store), `just deploy` (knarr →
  kai, kubs0). First consumer of the build-clones skill. korg program 1675.

## Done

- **001 — onto the kprojects harness.** Layout, `just check`, agent
  instruction files. No behaviour change.

## Next

- **CI.** The repo has no `.github/workflows/` at all, so the gate only runs
  when someone remembers. A workflow running `just check` on push/PR is the
  obvious next thing.

## Later / Ideas

- More commands. The framework is built for it — a command is one args struct,
  one `Handler`, one subcommand — but `launch-code` is still the only one.
- `rust-toolchain.toml` / `rustfmt.toml` / `clippy.toml`, mirrored from a
  sibling homelab repo, so the gate isn't at the mercy of a floating stable.
