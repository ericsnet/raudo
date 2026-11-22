# Measures Files' cold start and idle memory for comparison with Raudo.
# Run from an elevated-free session; it only starts and stops Files.

param(
    [int]$WaitSeconds = 30,
    [switch]$KeepOpen
)

$app = Get-StartApps | Where-Object { $_.Name -like 'Files*' } | Select-Object -First 1
if (-not $app) {
    Write-Error 'Files is not installed (Get-StartApps found no match).'
    exit 1
}

$running = Get-Process Files -ErrorAction SilentlyContinue
if ($running) {
    Write-Host ("Closing running Files (PID {0}) for a cold start..." -f $running.Id)
    $running | Stop-Process -Force
    Start-Sleep -Milliseconds 800
}

$sw = [Diagnostics.Stopwatch]::StartNew()
Start-Process ("shell:AppsFolder\{0}" -f $app.AppID)

$proc = $null
while ($sw.Elapsed.TotalSeconds -lt $WaitSeconds) {
    $proc = Get-Process Files -ErrorAction SilentlyContinue
    if ($proc -and $proc.MainWindowHandle -ne 0) { break }
    Start-Sleep -Milliseconds 50
}
$startup = [int]$sw.Elapsed.TotalMilliseconds
$sw.Stop()

Start-Sleep -Seconds 3
$proc = Get-Process Files -ErrorAction SilentlyContinue
$ram = if ($proc) { [math]::Round($proc.WorkingSet64 / 1MB) } else { 0 }

[pscustomobject]@{
    App          = 'Files'
    Version      = if ($proc) { $proc.ProductVersion } else { '' }
    StartupMs    = $startup
    StartupNote  = if ($proc -and $proc.MainWindowHandle -ne 0) { 'window shown' } else { 'timed out or no HWND' }
    IdleRamMB    = $ram
} | Format-List

if (-not $KeepOpen -and $proc) {
    $proc | Stop-Process -Force
}
