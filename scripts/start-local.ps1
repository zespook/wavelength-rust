$ErrorActionPreference = 'Stop'

$Root = Split-Path $PSScriptRoot -Parent
$QobuzDir = Join-Path $Root 'qobuz-rest-api'
$AudioDir = Join-Path $Root 'audio-service'
$FrontendDir = Join-Path $Root 'svelte-frontend'
$Python = (Get-Command python.exe -ErrorAction Stop).Source
$StartedServiceIds = @()
$rootEnvPath = Join-Path $Root '.env'
$rootEnv = if (Test-Path $rootEnvPath) { Get-Content $rootEnvPath } else { @() }

function Get-EnvValue([string]$Name) {
  $line = $rootEnv | Where-Object { $_ -match "^$Name=" } | Select-Object -First 1
  if (-not $line) { return '' }
  return (($line -split '=', 2)[1]).Trim().Trim('"')
}

function Test-TcpPort([int]$Port) {
  $client = [Net.Sockets.TcpClient]::new()
  try {
    $connection = $client.ConnectAsync('127.0.0.1', $Port)
    return $connection.Wait(500) -and $client.Connected
  } catch {
    return $false
  } finally {
    $client.Dispose()
  }
}

function Start-BackgroundService([string]$Name, [string]$WorkingDirectory, [int]$Port) {
  $logDirectory = Join-Path $env:LOCALAPPDATA 'Wavelength\dev-logs'
  New-Item -ItemType Directory -Path $logDirectory -Force | Out-Null
  $process = Start-Process -FilePath $Python -ArgumentList @(
    '-m', 'uvicorn', 'main:app', '--host', '127.0.0.1', '--port', "$Port"
  ) -WorkingDirectory $WorkingDirectory -WindowStyle Hidden -PassThru `
    -RedirectStandardOutput (Join-Path $logDirectory "$Name.out.log") `
    -RedirectStandardError (Join-Path $logDirectory "$Name.err.log")
  $script:StartedServiceIds += $process.Id

  $deadline = [DateTime]::UtcNow.AddSeconds(30)
  while ([DateTime]::UtcNow -lt $deadline) {
    if (Test-TcpPort $Port) { return }
    $process.Refresh()
    if ($process.HasExited) {
      throw "$Name exited during startup. Check logs in $logDirectory."
    }
    Start-Sleep -Milliseconds 250
  }

  Stop-Process -Id $process.Id -ErrorAction SilentlyContinue
  throw "$Name did not start listening on port $Port. Check logs in $logDirectory."
}

& $Python -c "import fastapi, uvicorn, spotipy, yt_dlp, SpotiFLAC, httpx, mutagen, dotenv" 2>$null
if ($LASTEXITCODE -ne 0) {
  $PythonUserBase = (& $Python -c "import site; print(site.USER_BASE)").Trim()
  $AudioVenv = Join-Path $PythonUserBase 'wavelength-audio-venv'
  $VenvPython = Join-Path $AudioVenv 'Scripts\python.exe'
  if (-not (Test-Path $VenvPython)) {
    & $Python -m venv $AudioVenv
    if ($LASTEXITCODE -ne 0) { throw 'Could not create the local audio-service virtual environment.' }
  }
  & $VenvPython -m pip install -r (Join-Path $AudioDir 'requirements.txt') -r (Join-Path $QobuzDir 'requirements.txt')
  if ($LASTEXITCODE -ne 0) { throw 'Could not install local audio-service requirements.' }
  $Python = $VenvPython
}

$env:SPOTIFY_CLIENT_ID = Get-EnvValue 'SPOTIFY_CLIENT_ID'
$env:SPOTIFY_CLIENT_SECRET = Get-EnvValue 'SPOTIFY_CLIENT_SECRET'
$env:ROOM_SERVER_PORT = Get-EnvValue 'ROOM_SERVER_PORT'
$env:QOBUZ_APP_ID = Get-EnvValue 'QOBUZ_APP_ID'
$env:QOBUZ_SECRET = Get-EnvValue 'QOBUZ_SECRET'
$env:QOBUZ_TOKEN = Get-EnvValue 'QOBUZ_AUTH_TOKEN'
$env:QOBUZ_API_BASE = 'https://www.qobuz.com/api.json/0.2'
$env:QOBUZ_API_URL = 'http://127.0.0.1:8000'
$env:DOWNLOAD_ROOT = './downloads'
$env:LOG_LEVEL = 'WARNING'

try {
  if (-not (Test-TcpPort 8000)) {
    Start-BackgroundService 'qobuz-api' $QobuzDir 8000
  }

  if (-not (Test-TcpPort 8787)) {
    Start-BackgroundService 'audio-service' $AudioDir 8787
  }
} catch {
  foreach ($processId in $StartedServiceIds) {
    Stop-Process -Id $processId -ErrorAction SilentlyContinue
  }
  throw
}

if (Test-TcpPort 5173) {
  Write-Host 'Vite is already listening at http://127.0.0.1:5173/' -ForegroundColor Green
  return
}

Push-Location $FrontendDir
try {
  node .\node_modules\vite\bin\vite.js --host=0.0.0.0 --port=5173 --strictPort
} finally {
  Pop-Location
  foreach ($processId in $StartedServiceIds) {
    Stop-Process -Id $processId -ErrorAction SilentlyContinue
  }
}