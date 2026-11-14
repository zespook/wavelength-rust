param([ValidateSet('dev', 'build')][string]$Action = 'dev')

$ErrorActionPreference = 'Stop'

$Root = Split-Path $PSScriptRoot -Parent
$TauriCli = Join-Path $Root 'node_modules\@tauri-apps\cli\tauri.js'
$Node = (Get-Command node.exe -ErrorAction Stop).Source

if (-not (Test-Path $TauriCli)) {
  throw "Tauri CLI was not found at $TauriCli."
}

Push-Location $Root
try {
  if ($Action -eq 'build') {
    $PackageServices = Join-Path $Root 'scripts\package-services.ps1'
    & $PackageServices
    if ($LASTEXITCODE -ne 0) {
      throw "Service packaging failed with exit code $LASTEXITCODE."
    }

    & $Node $TauriCli build --bundles=nsis
  } else {
    & $Node $TauriCli dev
  }

  if ($LASTEXITCODE -ne 0) {
    throw "Tauri $Action failed with exit code $LASTEXITCODE."
  }
} finally {
  Pop-Location
}
