# Desktop Client

The Windows app is a small Tauri client using the system WebView2 runtime. It opens the bundled player immediately and works with imported local music without a server. A server URL is optional and is used only for shared listening rooms. The bundled player has no general native filesystem or shell permissions.

## Self-Hosted Server

The server is optional; the desktop app works without it. See the Docker server setup in [README.md](./README.md) to enable shared listening and Spotify OAuth.

The desktop Spotify sign-in opens the system browser and returns through a short-lived, one-time local handoff; the Spotify callback URI remains the room server's configured callback.

Room host control uses a random bearer token saved in the app's local storage so it survives restarts; the server stores only its hash. Rooms created before host-token protection must be recreated by their host.

## Local Development

From `svelte-frontend`, run `npm run dev`. The Windows launcher starts the local Qobuz API on `127.0.0.1:8000`, the audio/SpotiFLAC service on `127.0.0.1:8787`, and Vite on port `5173`. It does not start MongoDB or the room API; follow the server setup in [README.md](./README.md) if you need shared listening or Spotify OAuth. For local OAuth, register `http://127.0.0.1:5050/api/spotify/callback` in the Spotify developer dashboard and set `SPOTIFY_REDIRECT_URI` to that value for the local room API; production should use its public callback URL. Spotify catalog search uses `SPOTIFY_CLIENT_ID` and `SPOTIFY_CLIENT_SECRET` from the repository `.env`; YouTube search does not require those credentials. If the system Python lacks the service dependencies, the launcher creates a virtual environment and installs both service requirements.

## Build the Installer

From `svelte-frontend`:

```powershell
npm install
npm run desktop:build
```

The NSIS installer is written under `svelte-frontend/src-tauri/target/release/bundle/nsis/`. WebView2 is not bundled, keeping the installer small; Windows downloads its bootstrapper during installation when the runtime is missing. The current build is unsigned; production distribution should use a Windows code-signing certificate.

## Room Playback

The host publishes the queue, current track ID, playback state, and playback clock through the room API; guests poll and synchronize playback. Local audio is uploaded only when the host adds it to the queue, then streamed by room members from the server. Uploaded audio is removed after seven days. Local files that are not queued remain on the device.