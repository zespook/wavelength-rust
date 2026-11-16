param([ValidateSet('dev', 'build')][string]$Action = 'dev')

$ErrorActionPreference = 'Stop'

$Root = Split-Path $PSScriptRoot -Parent
$Dioxus = (Get-Command dx.exe -ErrorAction Stop).Source
$Frontend = Join-Path $Root 'dioxus-frontend'
$BundleDir = Join-Path $Root 'dist\desktop'
$PackageServices = Join-Path $Root 'scripts\package-services.ps1'

Push-Location $Frontend
try {
  & $PackageServices
  if ($LASTEXITCODE -ne 0) {
    throw "Service packaging failed with exit code $LASTEXITCODE."
  }

  if ($Action -eq 'build') {
    $versionMatch = [regex]::Match((Get-Content (Join-Path $Frontend 'Cargo.toml') -Raw), '(?m)^version\s*=\s*"([^"]+)"')
    if (-not $versionMatch.Success) { throw 'Could not read the app version from Cargo.toml.' }
    $version = $versionMatch.Groups[1].Value
    $serviceArchive = Join-Path $Root "dist\release-assets\Wavelength_${version}_services_windows-x86_64.zip"
    $packageOutput = & python (Join-Path $Root 'scripts\package-compiled-services.py') (Join-Path $Frontend 'resources\services') $version $serviceArchive
    if ($LASTEXITCODE -ne 0) { throw 'Could not package the compiled services.' }
    $hashLine = $packageOutput | Where-Object { $_ -match '^sha256=[0-9a-fA-F]{64}$' } | Select-Object -First 1
    if (-not $hashLine) { throw 'Could not read the compiled service package checksum.' }
    $env:WAVELENGTH_SERVICES_SHA256 = $hashLine.Substring(7).ToLowerInvariant()
    Remove-Item Env:\WAVELENGTH_SERVICES_ARCHIVE -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $BundleDir | Out-Null
    & $Dioxus bundle --platform desktop --release --no-default-features --features desktop --package-types nsis --out-dir $BundleDir
    if ($LASTEXITCODE -ne 0) { throw 'Could not build the Windows installer.' }
    Copy-Item $serviceArchive (Join-Path $BundleDir 'Wavelength_services_windows-x86_64.zip') -Force
  } else {
    & $Dioxus serve --platform desktop --no-default-features --features desktop
  }

  if ($LASTEXITCODE -ne 0) {
    throw "Dioxus $Action failed with exit code $LASTEXITCODE."
  }
} finally {
  Pop-Location
}
