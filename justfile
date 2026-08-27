# Windows: `just` runs recipes through `sh`, which Windows does not ship — put
# Git for Windows' `usr\bin` on PATH (it holds `sh.exe`) or run from Git Bash.
# (Upstream's own requirement: "sh must be available in the PATH".)

# List available recipes
default:
    @just --list

# Run CI gates (lint, typecheck, tests)
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test

# Apply formatting
fmt:
    cargo fmt

# The store label `just publish` would use for the current checkout.
#
# build.rs emits the label verbatim as the second field, so this reads it
# rather than reassembling it — see the comment there for why that matters.
[doc("Show the store label this checkout would publish under")]
version:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --release -q -p krcmd
    ./target/release/krcmd --version | awk '{ print $2 }'

# Publish the Linux client to the homelab package store (kubsdb :4880).
#
# CLIENT ONLY, deliberately: `krcmd-host.exe` runs on cleo (Windows), and
# store-distribution of Windows binaries waits on a Windows install story
# (kpolice's install-cleo.ps1 is the stand-in precedent; a second repo
# needing it is the signal to ask knarr for Windows support). When that
# lands, add the `x86_64-pc-windows-gnu` build here as a second artifact in
# the SAME version — never as a second version (see kpolice's justfile).
#
# Runs on a Linux host (kai, via the build-clones skill) — the canonical
# clone lives on cleo, which cannot build Linux binaries. The binary is
# re-read with `--version` and published under the label that stamp
# produces, so the stamp and the store label are one fact; that read is the
# same command knarr's confirm step runs on the target.
[doc("Publish the Linux client binary to the package store")]
publish:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "publish: refusing to publish from a dirty tree — a published version must name a commit" >&2
        exit 1
    fi
    cargo build --release -p krcmd
    stamp="$(./target/release/krcmd --version)"
    v="$(printf '%s\n' "$stamp" | awk '{ print $2 }')"
    case "$v" in
        *dirty*|*unknown*)
            echo "publish: binary stamped '$stamp' — that names no reproducible commit" >&2
            exit 1 ;;
    esac
    # A branch commit vanishes from history at squash-merge, so a branch
    # build may exist in the store to prove a path, but must never become
    # what the fleet resolves as `latest`.
    latest_arg=""
    if [[ "$(git rev-parse --abbrev-ref HEAD)" != "main" ]]; then
        latest_arg="--no-latest"
        echo "publish: not on main — publishing $v WITHOUT moving the latest pointer" >&2
    fi
    arch="$(uname -m)-$(uname -s | tr '[:upper:]' '[:lower:]')"
    echo "==> publishing krcmd $v as $stamp (linux client)"
    d=$(ssh -n kubsdb mktemp -d)
    scp target/release/krcmd kubsdb:"$d/krcmd-$arch"
    ssh -n kubsdb "kpkg artifact $latest_arg krcmd $v $d/* && rm -rf $d"

# Deploy the store's `latest` to the Linux fleet.
#
# knarr's defaults are already correct for this shape: --dest
# /usr/local/bin/krcmd, --file krcmd-x86_64-linux, and no --unit (a CLI,
# nothing to restart). --store is passed explicitly because KNARR_STORE_URL
# is set nowhere on the fleet (recorded gotcha). Pass --version to pin a
# branch build, or --dry-run to see the plan.
#
# cleo is NOT here: it runs the host daemon, not this client, and knarr's
# install step is `install -m 0755` over ssh — not the Windows shape.
[doc("Deploy the store's latest client to the Linux fleet (kai, kubs0)")]
deploy *ARGS:
    knarr deploy krcmd --host kai,kubs0 --store https://kubsdb.encke-wahoo.ts.net:4880 {{ARGS}}
