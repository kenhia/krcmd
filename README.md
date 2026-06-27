# krcmd — Ken's Remote Command

Launch GUI tools on your **host** machine from a **remote** dev box you're SSH'd
into, over a tiny signed HTTP protocol. The flagship use: from `kai`, in the
directory you're already working in, run

```console
$ krcmd vsci .
```

and VS Code **Insiders** opens on your Windows host, connected straight back to
`ken@kai:/home/ken/...` via the Remote-SSH extension. Use `vsc` for stable VS Code.

This is a from-scratch successor to [`krrr`](https://github.com/kenhia/krrr),
adding three things it lacked:

1. **Authentication** — every request is signed with your existing SSH key and
   verified against a trust list. No more open door.
2. **A command framework** — the host runs only a fixed set of *named, typed*
   commands. There is deliberately no way to run ad-hoc/arbitrary commands.
3. **VS Code + Insiders** — pick the flavor per invocation.

## How it works

```
  remote box (kai)                         host (Windows)
  ────────────────                         ──────────────
  krcmd vsci .                             krcmd-host (daemon)
    │  build {command, args, ts, nonce}      │
    │  sign with ~/.ssh/id_ed25519           │
    │  POST /command  ───────────────────►   │  verify timestamp window
    │                                        │  verify SSHSIG vs allowed_signers
    │                                        │  reject replayed nonce
    │                                        │  dispatch to a registered handler
    │   ◄───────────────────  JSON result    │  → launch code-insiders --folder-uri …
```

**Auth** reuses your SSH keys via the SSHSIG signature format (the same thing
`ssh-keygen -Y sign` produces), implemented in pure Rust with the
[`ssh-key`](https://crates.io/crates/ssh-key) crate — no subprocess, no new key
material. The host trusts a list of public keys keyed by identity
(`allowed_signers`), exactly analogous to `~/.ssh/authorized_keys`. Replay is
prevented by a signed timestamp (a freshness window) plus a per-nonce cache.

## Layout

A Cargo workspace with two binaries over one shared protocol crate:

| crate         | what it is                                                        |
|---------------|-------------------------------------------------------------------|
| `krcmd-proto` | shared wire types, SSHSIG sign/verify, `allowed_signers`, command arg structs |
| `krcmd-host`  | the daemon (runs on the host); registry + command handlers        |
| `krcmd`       | the remote CLI (runs on the dev box); signs and sends requests     |

## Build

```console
$ cargo build --release
# binaries land in target/release/{krcmd-host.exe, krcmd}
```

Requires an **unencrypted** ed25519 key (`~/.ssh/id_ed25519`). krcmd does not
prompt for passphrases; if your key is passphrase-protected, point `--key` at an
unencrypted one.

## Host setup (Windows)

1. Copy `krcmd-host.example.toml` to a place the daemon searches — `next to the
   exe` (e.g. `C:\tools\bin\krcmd-host.toml`), `%APPDATA%\krcmd\krcmd-host.toml`,
   or wherever `$KRCMD_HOST_CONFIG` points — and set the VS Code launcher paths.
   The startup banner prints which config file was loaded (or that none was
   found), plus the resolved `allowed_signers` and VS Code paths — check it if a
   setting seems ignored.
2. Create the trust list. On each dev box, print its entry:
   ```console
   $ echo "$USER@$(hostname) $(cat ~/.ssh/id_ed25519.pub)"
   ```
   Paste the lines into the `allowed_signers` file referenced by the config (see
   `example.allowed_signers`).
3. Run the daemon:
   ```console
   > krcmd-host.exe
   krcmd-host
     bind            0.0.0.0:42271
     allowed signers 2 (C:\Users\you\.krcmd\allowed_signers)
     commands        launch-code
     max skew        60s
   ready: listening on http://0.0.0.0:42271
   ```
   Add `--dry-run` (or `KRCMD_DRY_RUN=1`) to have handlers describe what they
   would do without executing — handy while wiring things up.

## Remote setup (dev box)

Put `krcmd` on your `PATH`. With no flags it figures out:

- **which host to call** — the first field of `$SSH_CLIENT` (the box you SSH'd in
  from), port `42271`. Override with `--server host[:port]` / `$KRCMD_HOST`.
- **identity** — `<user>@<hostname>`, matched against `allowed_signers`. Override
  with `--identity` / `$KRCMD_IDENTITY`.
- **ssh host the host reconnects to** — this box's `hostname`. If the host
  reaches this box under a different SSH alias, set `--host alias` /
  `$KRCMD_SSH_HOST`.
- **key** — `~/.ssh/id_ed25519`. Override with `--key` / `$KRCMD_KEY`.

## Usage

```console
$ krcmd vsci .                 # open Insiders at the current dir
$ krcmd vsc  .                 # open stable VS Code
$ krcmd vsci ~/src/widget      # open a specific dir
$ krcmd vsci . --dry-run       # print the signed request, don't send
$ krcmd vsc . --host kai-lan   # tell the host to reconnect via a different alias
```

### fish

The binary already derives everything from `$SSH_CLIENT`, so no wrapper function
is needed — just run `krcmd vsci .`. If a box needs a non-default alias or server,
set it once in `config.fish`:

```fish
set -gx KRCMD_SSH_HOST kai
# set -gx KRCMD_HOST 192.168.1.19   # only if SSH_CLIENT isn't usable
```

## Adding a command

The framework keeps this to one module + one line:

1. Add the command name + a typed args struct in
   `crates/krcmd-proto/src/commands.rs` (shared so the client builds it too).
2. Add a handler in `crates/krcmd-host/src/commands/`, implementing `Handler`
   (`name()` + `run(payload, ctx)`); deserialize the payload into your args,
   validate, and execute. Register it in `build_registry()` in `main.rs`.
3. Add a subcommand to `crates/krcmd/src/main.rs` that builds the payload.

The host will only ever run commands that have a registered handler, invoked with
validated arguments passed directly to the OS (never through a shell).

## Security notes

- Intended for a trusted LAN. Bind to a specific interface and don't expose the
  port to the internet — signatures authenticate requests but the service is not
  hardened for hostile networks.
- Signature verification happens **before** the nonce is recorded, so bogus
  requests can't poison the replay cache.
- Handlers validate their inputs (e.g. `launch-code` requires an absolute path
  and a restricted charset for user/host) even though arguments never touch a
  shell.

## License

MIT © Ken Hiatt
