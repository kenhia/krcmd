# install-autostart-cleo.ps1 — register the krcmd-host logon autostart task.
#
# Registers a Scheduled Task that starts krcmd-host at kenhi's logon, in the
# INTERACTIVE session (the daemon launches VS Code onto the desktop, so a
# service or a run-whether-logged-on task would be wrong), hidden, with
# stdout/stderr captured to ~/.config/ — the exact detached-start pattern
# documented in docs/host-cleo.md, made durable across reboots.
#
# Idempotent: re-running replaces the existing task. Run as kenhi on cleo
# (an elevated ssh session works; the task principal is pinned to kenhi's
# interactive logon either way). korg WI 1677.
#
# The wrapper indirection matters: a task action cannot redirect output
# itself, and krcmd-host logs only to stdout. The hidden PowerShell wrapper
# Start-Process-es the daemon detached with both streams redirected, then
# exits — so the task shows "completed" seconds after logon while the daemon
# keeps running.

[CmdletBinding()]
param(
    [string]$TaskName = 'krcmd-host',
    [string]$Exe      = 'C:\tools\bin\krcmd-host.exe',
    [string]$LogDir   = 'C:\Users\kenhi\.config'
)

$ErrorActionPreference = 'Stop'

if (-not (Test-Path $Exe)) { throw "krcmd-host not found at $Exe" }
if (-not (Test-Path $LogDir)) { New-Item -ItemType Directory -Force $LogDir | Out-Null }

$wrapped = "Start-Process -FilePath '$Exe' -WindowStyle Hidden " +
           "-RedirectStandardOutput '$LogDir\krcmd-host.log' " +
           "-RedirectStandardError '$LogDir\krcmd-host.err.log'"

$action    = New-ScheduledTaskAction -Execute 'powershell.exe' `
                 -Argument "-NoProfile -WindowStyle Hidden -Command `"$wrapped`""
$trigger   = New-ScheduledTaskTrigger -AtLogOn -User 'kenhi'
$principal = New-ScheduledTaskPrincipal -UserId 'kenhi' -LogonType Interactive
# Defaults that would bite: a 3-day ExecutionTimeLimit would kill the wrapper's
# task record harmlessly but reads as failure; battery rules don't apply to a
# desktop but cost nothing to disarm.
$settings  = New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 5) `
                 -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
                 -MultipleInstances IgnoreNew

if (Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue) {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
}
Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger `
    -Principal $principal -Settings $settings | Out-Null

$task = Get-ScheduledTask -TaskName $TaskName
Write-Host "registered : $($task.TaskName) (state: $($task.State))"
Write-Host "trigger    : at logon of kenhi, interactive session"
Write-Host "action     : $Exe (hidden, logs in $LogDir)"
Write-Host ""
Write-Host "Start now without waiting for a logon:  Start-ScheduledTask -TaskName $TaskName"
Write-Host "(stop any already-running krcmd-host first: Get-Process krcmd-host | Stop-Process)"
