param([ValidateSet('dev', 'build')][string]$Action = 'dev')

$ErrorActionPreference = 'Stop'

$Root = Split-Path $PSScriptRoot -Parent
$Dioxus = (Get-Command dx.exe -ErrorAction Stop).Source
$Frontend = Join-Path $Root 'dioxus-frontend'
$BundleDir = Join-Path $Root 'dist\desktop'

Push-Location $Frontend
try {
  if ($Action -eq 'build') {
    $PackageServices = Join-Path $Root 'scripts\package-services.ps1'
    & $PackageServices
    if ($LASTEXITCODE -ne 0) {
      throw "Service packaging failed with exit code $LASTEXITCODE."
    }

    New-Item -ItemType Directory -Force -Path $BundleDir | Out-Null
    & $Dioxus bundle --platform desktop --release --no-default-features --features desktop --package-types nsis --out-dir $BundleDir
  } else {
    $PackageServices = Join-Path $Root 'scripts\package-services.ps1'
    & $PackageServices
    if ($LASTEXITCODE -ne 0) {
      throw "Service packaging failed with exit code $LASTEXITCODE."
    }

    & $Dioxus serve --platform desktop --no-default-features --features desktop
  }

  if ($LASTEXITCODE -ne 0) {
    throw "Dioxus $Action failed with exit code $LASTEXITCODE."
  }
} finally {
  Pop-Location
}
