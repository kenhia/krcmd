# 001 — onto the kprojects harness

korg: proposal 1275, work item [#1272](https://github.com/kenhia/krcmd) · chore · 2026-08-15

## Goal

Put krcmd on the [kprojects](https://github.com/kenhia/kprojects) minimal
harness. It had no harness at all: no `justfile`, no `.github`, no agent
instruction files. Batch 5 of the fleet-wide rollout (korg #737).

Layout-only by intent — nothing about how krcmd behaves should change.

## What shipped

- `kproject-install --agent both .` — stack detected as **rust** from the root
  `Cargo.toml`, no `--stack` override needed.
- Seeded `sprints/{planning,review}`, `docs/`, `.scratch/`, and the rust
  `justfile` (`fmt --check` / `clippy --all-targets -D warnings` / `test`).
- Managed block created fresh in both `CLAUDE.md` and
  `.github/copilot-instructions.md`, plus a hand-written `## Project` section
  outside the markers covering the three crates, the design invariants worth
  not eroding, and the secret discipline around the example files.
- `.gitignore`: `.scratch/` and `target/` from the installer, and
  `.korg-sprint-proposal` for the sprint marker.

## Decisions

**D-1 — migrate cleo's clone, drop kai's.** korg #737's census deferred which
copy was canonical; measurement settled it. cleo `D:\ClaudeWorks\krcmd` had 3
commits and matched `origin/main`; kai `~/src/tools/krcmd` had 1 (`63a7fa8`,
the initial) and was two behind on the same origin. Nothing existed only on
kai, so that clone was removed rather than pulled — a stale copy that *also*
lacks the harness is precisely the drift #737 exists to end.

**D-2 — ran `cargo fmt` even though a failing gate is normally a stop.** The
proposal said to park a gate that fails on pre-existing lint, because fixing
lint is not a layout-only chore. The gate did fail, but only on
`cargo fmt --check`, and only one method chain in `krcmd-host/src/config.rs`
(`exe_dir`, reflowed across four lines). Clippy `-D warnings` came back clean
across all targets and all 14 tests passed. A rustfmt reflow is whitespace and
provably behaviour-preserving, and the harness ships `just fmt` for it — so it
was treated as inside the chore. The stop condition's stated rationale is
behaviour change, and there is none here.

The substantive worry — that a first `-D warnings` run on never-gated code
would light up — did not materialise. The gate is honest and it is green.

## Follow-ups

- **No CI** — korg #1285. There is still no `.github/workflows/`, so
  `just check` only runs when someone remembers, which is half a gate. Adding
  a workflow was out of scope for a layout chore. Worth pinning the toolchain
  in the same pass, since `@stable` floats.
- **kai's clone was already gone.** D-1 called for pulling or dropping it, but
  a sweep of kai and kubs0 found no krcmd checkout at all — `~/src/tools/krcmd`
  no longer exists. cleo's is the only copy, so nothing needed deleting.
- `docs/` and `.scratch/` are seeded but empty, so git doesn't carry them; a
  fresh clone won't have them until something lands inside. Harness-wide
  behaviour, not a krcmd problem.
