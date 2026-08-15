<!-- kproject:begin — managed by kprojects; do not edit inside this block -->
## kproject conventions

This project uses the kproject minimal harness
(<https://github.com/kenhia/kprojects>). Keep context small; prefer doing
over ceremony.

### Layout

- `sprints/` — the project's evolution, one record per PR-sized unit of
  work (a "sprint")
  - `planning/` — planning docs; at minimum `roadmap.md` (the general plan)
  - `review/` — more formal reviews as the project matures
  - sprint records: `###-<short-name>.md` for small projects, or a
    `###-<short-name>/` directory of files for larger/more formal ones
  - a sprint record is one informal narrative: goal, decisions, what
    shipped, follow-ups — written during the sprint, not after
- `docs/` — project documentation, architecture, usage
- `.scratch/` — git-ignored scratch space for user or agent ephemera;
  use it instead of /tmp
- `justfile` — dev recipes; default recipe is `@just --list`; `just check`
  runs the CI gates; `just deploy` (or variants) if the project deploys
- `.env` — git-ignored; tokens and environment vars

### Workflow

- One sprint ≈ one PR. Sprint proposals and work items are managed in
  `korg`; durable cross-project knowledge goes in `klams`.
- Mark each work item resolved as its work completes — don't batch the
  resolutions into sprint-ship. A proposal's progress should be readable
  while the sprint is running, which is the only time it is useful.
- If the korg or klams MCP tools are unavailable in your session, say so
  up front — don't silently work around missing infrastructure.
- TDD preferred: write the failing test first when practical.

### Tooling preferences

- Rust managed by `cargo`; format with `cargo fmt`, lint with
  `cargo clippy --all-targets` (test targets included deliberately — a gate
  that skips them is a gate that lies)
- Mirror `rust-toolchain.toml`, `rustfmt.toml` and `clippy.toml` from a
  sibling homelab repo rather than generating them
- License is MIT unless specifically directed otherwise
<!-- kproject:end -->

## Project

**krcmd — Ken's Remote Command.** Launch GUI tools on a *host* machine from a
*remote* dev box you're SSH'd into, over a small signed HTTP protocol. The
flagship use: `krcmd vsci .` on kai opens VS Code Insiders on the Windows host,
reconnected to `ken@kai:/home/ken/...` via Remote-SSH.

Successor to [`krrr`](https://github.com/kenhia/krrr), adding the three things
it lacked: authentication, a fixed command framework, and Insiders support.

### Layout — a Cargo workspace, two binaries over one protocol crate

| crate | what it is |
|---|---|
| `crates/krcmd-proto` | shared wire types, SSHSIG sign/verify, `allowed_signers` parsing, typed command args |
| `crates/krcmd-host` | the daemon, runs on the **host** (Windows); registry + handlers |
| `crates/krcmd` | the CLI, runs on the **dev box**; signs and sends requests |

Mind the direction: `krcmd-host` runs on cleo, `krcmd` runs on kai/kubs0. This
repo's working copy lives on the host side (`cleo:D:\ClaudeWorks\krcmd`).

### Design invariants — don't erode these

- **No ad-hoc commands, ever.** The host runs only *named, typed* commands with
  a registered handler. There is deliberately no path to arbitrary execution.
- **Arguments never touch a shell** — they're passed directly to the OS.
  Handlers validate anyway (`launch-code` requires an absolute path and a
  restricted charset for user/host); keep that belt-and-braces.
- **Signature verification precedes nonce recording**, so bogus requests can't
  poison the replay cache. Preserve that ordering.
- Auth reuses existing SSH keys via SSHSIG (pure Rust, `ssh-key` crate) — no
  subprocess, no new key material.
- Trusted-LAN threat model. Signatures authenticate; the service is not
  hardened for hostile networks.

Adding a command touches exactly three places: the name + args struct in
`krcmd-proto/src/commands.rs`, a `Handler` in `krcmd-host/src/commands/`
registered in `build_registry()`, and a subcommand in `krcmd/src/main.rs`.

### Secret discipline

- `example.allowed_signers` and `krcmd-host.example.toml` are **examples**.
  They must never acquire a real signer list, real paths, or real values.
- Live config is per-user and out of tree: `~/.config/krcmd-host.toml` (see the
  example file for the full search order). The real trust list is
  `~/.krcmd/allowed_signers`.
- `.gitignore` excludes `allowed_signers` while whitelisting the example — keep
  that pairing intact.
- krcmd requires an **unencrypted** ed25519 key; it never prompts for a
  passphrase.
