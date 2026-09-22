# Ship MSI post-install — invoked via WiX CustomAction after the MSI
# drops `C:\Program Files\Ship\shipctl.exe` + `shipd.exe`.
#
# cargo-dist's WiX template surfaces this via an install-path relative
# reference; the ExecAction carries an elevated impersonated user token
# so the scheduled task lands in the user's context, not SYSTEM's.

$ErrorActionPreference = 'Stop'

$installRoot = $env:INSTALLDIR
if (-not $installRoot) {
    $installRoot = (Get-Item -Path $PSCommandPath).Directory.Parent.FullName
}

$shipctl = Join-Path $installRoot 'shipctl.exe'
$shipd   = Join-Path $installRoot 'shipd.exe'

if (-not (Test-Path $shipctl) -or -not (Test-Path $shipd)) {
    Write-Warning "[ship] shipctl.exe or shipd.exe not found under '$installRoot'; skipping service registration."
    exit 0
}

try {
    & $shipctl daemon install-service --shipd-path $shipd
    Write-Output "[ship] daemon service registered."
} catch {
    Write-Warning "[ship] shipctl daemon install-service failed: $_"
    # Don't fail the installer — autostart can be re-run manually.
    exit 0
}
