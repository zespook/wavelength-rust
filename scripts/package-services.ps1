$ErrorActionPreference = 'Stop'

$RootDir = Split-Path $PSScriptRoot -Parent
$AudioDir = Join-Path $RootDir 'audio-service'
$QobuzDir = Join-Path $RootDir 'qobuz-rest-api'
$ResourceDir = Join-Path $RootDir 'dioxus-frontend\resources\services'
$BundleResourceDir = Join-Path $RootDir 'dioxus-frontend\resources'
$BuildRoot = Join-Path $env:TEMP 'wavelength-desktop-service-build'
$BuildVenv = Join-Path $env:TEMP 'wavelength-service-build-venv'
$VenvPython = Join-Path $BuildVenv 'Scripts\python.exe'

if (-not (Test-Path $VenvPython)) {
    $Python = (Get-Command python.exe -ErrorAction Stop).Source
    & $Python -m venv $BuildVenv
    if ($LASTEXITCODE -ne 0) { throw 'Could not create the service packaging environment.' }
}

& $VenvPython -m pip install --disable-pip-version-check -r (Join-Path $AudioDir 'requirements.txt') -r (Join-Path $QobuzDir 'requirements.txt') 'pyinstaller==6.12.0'
if ($LASTEXITCODE -ne 0) { throw 'Could not install service packaging requirements.' }

$NodeCommand = Get-Command node.exe -All -ErrorAction Stop |
    Where-Object { $_.Source -and -not $_.Source.StartsWith($ResourceDir, [StringComparison]::OrdinalIgnoreCase) } |
    Select-Object -First 1
if (-not $NodeCommand) { throw 'A system Node.js executable is required to build the desktop bundle.' }

if (Test-Path $BuildRoot) { Remove-Item $BuildRoot -Recurse -Force }
if (Test-Path $ResourceDir) { Remove-Item $ResourceDir -Recurse -Force }
$ServiceDist = Join-Path $BuildRoot 'dist'
$ServiceWork = Join-Path $BuildRoot 'work'
$ServiceSpec = Join-Path $BuildRoot 'spec'
$ApiResourceDir = Join-Path $ResourceDir 'api'
$RuntimeResourceDir = Join-Path $ResourceDir 'runtime'
New-Item -ItemType Directory -Force -Path $ServiceDist, $ServiceWork, $ServiceSpec, $ApiResourceDir, $RuntimeResourceDir | Out-Null

$ServiceArgs = @(
    '--noconfirm', '--clean', '--onedir',
    '--name', 'WavelengthLocalService',
    '--distpath', $ServiceDist,
    '--workpath', $ServiceWork,
    '--specpath', $ServiceSpec,
    '--paths', $AudioDir,
    '--paths', $QobuzDir,
    '--add-data', "$(Join-Path $AudioDir 'main.py');audio",
    '--add-data', "$(Join-Path $AudioDir 'playlist.py');audio",
    '--add-data', "$(Join-Path $QobuzDir 'main.py');qobuz",
    '--hidden-import', 'playlist',
    '--collect-all', 'fastapi',
    '--collect-all', 'uvicorn',
    '--collect-all', 'starlette',
    '--collect-all', 'pydantic',
    '--collect-all', 'httpx',
    '--collect-all', 'spotipy',
    '--collect-all', 'SpotiFLAC',
    '--collect-all', 'yt_dlp',
    '--collect-all', 'imageio_ffmpeg',
    '--collect-all', 'packaging',
    '--collect-all', 'mutagen',
    '--collect-all', 'certifi',
    '--copy-metadata', 'SpotiFLAC',
    (Join-Path $AudioDir 'desktop_service.py')
)
& $VenvPython -m PyInstaller @ServiceArgs
if ($LASTEXITCODE -ne 0) { throw 'Could not package the local audio and Qobuz services.' }
Copy-Item (Join-Path $ServiceDist 'WavelengthLocalService\*') $ApiResourceDir -Recurse -Force
Copy-Item $NodeCommand.Source (Join-Path $RuntimeResourceDir 'node.exe') -Force

if (-not (Test-Path (Join-Path $ApiResourceDir 'WavelengthLocalService.exe'))) {
    throw 'The combined local service executable is missing.'
}
if (-not (Test-Path (Join-Path $RuntimeResourceDir 'node.exe'))) {
    throw 'The bundled Node.js runtime is missing.'
}

Write-Host 'Combined audio/Qobuz service and Node.js runtime are ready for desktop development.' -ForegroundColor Green