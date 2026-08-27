# krcmd-host on cleo — how it actually runs

State as found and configured 2026-08-27 (korg WI 1672).

## The pieces

| what | where |
|---|---|
| daemon binary | `C:\tools\bin\krcmd-host.exe` |
| live config | `C:\Users\kenhi\.config\krcmd-host.toml` |
| trust list | `C:\Users\kenhi\.ssh\allowed_signers` (`ken@kai`, `ken@kubs0`) |
| bind | `0.0.0.0:42271` |

The trust list is read **at startup only** — adding a signer requires a
daemon restart. The startup banner prints the resolved config path, the
signer count, and the VS Code launcher paths; check it whenever a setting
seems ignored.

## Launch: scheduled task at logon

Since sprint 003 (korg WI 1677) the daemon autostarts via the Scheduled
Task **`krcmd-host`**: at kenhi's logon, in the interactive session (it
must be — the daemon launches VS Code onto the desktop, so a service or a
run-whether-logged-on task would be wrong), hidden, stdout/stderr to
`~/.config/krcmd-host.log` / `.err.log`. Registered (idempotently) by
`scripts/install-autostart-cleo.ps1`; re-run that script after changing
the exe path or log location.

History: before 2026-08-27 there was no autostart at all — the daemon was
found *not running*, meaning `krcmd` from kai had been dead-at-rest for
some time. That manual-launch era is why this section exists.

To restart after a trust-list or config change (the list is read at
startup only):

```powershell
Get-Process krcmd-host -ErrorAction SilentlyContinue | Stop-Process
Start-ScheduledTask -TaskName krcmd-host
```

To start it by hand on a box without the task, detached with output
captured:

```powershell
Start-Process -FilePath 'C:\tools\bin\krcmd-host.exe' -WindowStyle Hidden `
  -RedirectStandardOutput 'C:\Users\kenhi\.config\krcmd-host.log' `
  -RedirectStandardError  'C:\Users\kenhi\.config\krcmd-host.err.log'
```

Verify (either path): `Get-NetTCPConnection -LocalPort 42271 -State Listen`,
and the banner in the `.log` file should count the expected signers.

## Client fleet (for context)

The client is store-published (`just publish`, from the kai build-clones
cache) and knarr-deployed to `/usr/local/bin/krcmd` on kai and kubs0 —
see `sprints/002-store-publish.md`. cleo runs only the host daemon.
