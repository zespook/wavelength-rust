$envFile = Join-Path (Split-Path $PSScriptRoot -Parent) '.env'
Get-Content $envFile | Where-Object { $_ -match '^(PORT|MONGODB_URI|AUDIO_SERVICE_URL|QOBUZ_API_URL|PASSCODE|ENCRYPTION_KEY|QOBUZ_APP_ID|QOBUZ_SECRET|QOBUZ_AUTH_TOKEN)=' } | ForEach-Object {
  $k = ($_ -split '=')[0]
  $v = ($_ -split '=',2)[1]
  if ($k -eq 'PASSCODE' -or $k -eq 'ENCRYPTION_KEY' -or $k -eq 'QOBUZ_SECRET' -or $k -eq 'QOBUZ_AUTH_TOKEN') {
    "$k=<set>"
  } else {
    "$k=$v"
  }
}
