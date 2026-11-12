# Desktop Client

The Windows app is a small Tauri client using the system WebView2 runtime. It opens the bundled player immediately and works with imported local music without a server. A server URL is optional and is used only for shared listening rooms. The bundled player has no general native filesystem or shell permissions.

## Self-Hosted Server

To enable shared listening, start the room-only API on the host machine:

```powershell
docker compose up -d --build
```

The server has no WebUI. It exposes the room and Spotify APIs while MongoDB remains private to the Compose network. Copy `.env.example` to `.env` and set `ROOM_SERVER_PASSCODE` to a unique, high-entropy value of at least 16 characters; the API refuses to start without it and verifies it with bcrypt. Passcode authentication is limited to 8 attempts per client IP per minute. New room codes are 12 characters; room joins are limited to 10 attempts per IP per minute, and room reads and public audio streams to 60 per IP per minute. Existing 4–6 character room codes remain valid for compatibility, so close and recreate older rooms to get a new 12-character code. Clients enter the server passcode in Settings when connecting. The desktop app saves it in the operating system's credential vault; the browser app saves it in browser local storage, which is accessible to scripts running on that origin. On startup, the client exchanges the saved passcode for a short-lived access token kept only in app memory; room REST requests, Socket.IO connections, and shared-audio streams require authorization. Set `ROOM_SERVER_PORT` to change the host port (default `5050`). In the app, save `http://localhost:5050` for same-device use, or the HTTPS URL of your reverse proxy for remote clients. Use HTTPS for remote connections. If a reverse proxy sits in front of the API, set `ROOM_SERVER_TRUSTED_PROXIES` to its peer IP address(es) and configure it to overwrite `X-Real-IP`; otherwise all proxied clients share the proxy's rate-limit bucket. Spotify OAuth requires `SPOTIFY_CLIENT_ID`, `SPOTIFY_CLIENT_SECRET`, and `SPOTIFY_REDIRECT_URI` in the Compose environment; register the redirect URI in the Spotify developer dashboard. Set `FRONTEND_URL` to the public frontend origin when it differs from the redirect URI's origin. The app itself remains usable when this setting is empty.

The desktop Spotify sign-in opens the system browser and returns through a short-lived, one-time local handoff; the Spotify callback URI remains the room server's configured callback.

Room host control uses a random bearer token saved in the app's local storage so it survives restarts; the server stores only its hash. Rooms created before host-token protection must be recreated by their host.

## Local Development

From `svelte-frontend`, run `npm run dev`. The Windows launcher starts the local Qobuz API on `127.0.0.1:8000`, the audio/SpotiFLAC service on `127.0.0.1:8787`, and Vite on port `5173`. It does not start MongoDB or the room API; start the Compose server above to use Spotify OAuth and library sync. For local OAuth, register `http://127.0.0.1:5050/api/spotify/callback` in the Spotify developer dashboard and set `SPOTIFY_REDIRECT_URI` to that value for the local Room API; production should use its public callback URL. Spotify catalog search uses `SPOTIFY_CLIENT_ID` and `SPOTIFY_CLIENT_SECRET` from the repository `.env`; YouTube search does not require those credentials. If the system Python lacks the service dependencies, the launcher creates a virtual environment and installs both service requirements.

## Build the Installer

From `svelte-frontend`:

```powershell
npm install
npm run desktop:build
```

The NSIS installer is written under `svelte-frontend/src-tauri/target/release/bundle/nsis/`. WebView2 is not bundled, keeping the installer small; Windows downloads its bootstrapper during installation when the runtime is missing. The current build is unsigned; production distribution should use a Windows code-signing certificate.

## Room Playback

The host publishes the queue, current track ID, playback state, and playback clock through the room API; guests poll and synchronize playback. Local audio is uploaded only when the host adds it to the queue, then streamed by room members from the server. Uploaded audio is removed after seven days. Local files that are not queued remain on the device.