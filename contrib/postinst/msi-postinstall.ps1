# Ship MSI post-install — invoked via WiX CustomAction after the MSI
# drops `C:\Program Files\Ship\ship.exe` + `shipd.exe`.
#
# cargo-dist's WiX template surfaces this via an install-path relative
# reference; the ExecAction carries an elevated impersonated user token
# so the scheduled task lands in the user's context, not SYSTEM's.

$ErrorActionPreference = 'Stop'

$installRoot = $env:INSTALLDIR
if (-not $installRoot) {
    $installRoot = (Get-Item -Path $PSCommandPath).Directory.Parent.FullName
}

$ship  = Join-Path $installRoot 'ship.exe'
$shipd = Join-Path $installRoot 'shipd.exe'

if (-not (Test-Path $ship) -or -not (Test-Path $shipd)) {
    Write-Warning "[ship] ship.exe or shipd.exe not found under '$installRoot'; skipping service registration."
    exit 0
}

try {
    & $ship daemon install-service --shipd-path $shipd
    Write-Output "[ship] daemon service registered."
} catch {
    Write-Warning "[ship] ship daemon install-service failed: $_"
    # Don't fail the installer — autostart can be re-run manually.
    exit 0
}
