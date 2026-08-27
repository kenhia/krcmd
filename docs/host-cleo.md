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

## Launch: manual, no autostart (a known gap)

There is **no autostart**: not in HKCU/HKLM `Run`, not a service, not a
scheduled task, not the Startup folder (all four checked 2026-08-27). The
daemon runs only when someone starts it — and it was found *not running*,
meaning `krcmd` from kai had been dead-at-rest for some time. An autostart
(likely a logon-triggered scheduled task) is an open work item in the korg
`krcmd` project.

Until that lands, start it detached with output captured:

```powershell
Start-Process -FilePath 'C:\tools\bin\krcmd-host.exe' -WindowStyle Hidden `
  -RedirectStandardOutput 'C:\Users\kenhi\.config\krcmd-host.log' `
  -RedirectStandardError  'C:\Users\kenhi\.config\krcmd-host.err.log'
```

Verify: `Get-NetTCPConnection -LocalPort 42271 -State Listen`, and the
banner in the `.log` file should count the expected signers.

To restart (after a trust-list or config change):

```powershell
Get-Process krcmd-host | Stop-Process
# then the Start-Process line above
```

## Client fleet (for context)

The client is store-published (`just publish`, from the kai build-clones
cache) and knarr-deployed to `/usr/local/bin/krcmd` on kai and kubs0 —
see `sprints/002-store-publish.md`. cleo runs only the host daemon.
