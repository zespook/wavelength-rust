# Desktop Client

The Windows app is a small Tauri client using the system WebView2 runtime. It opens the bundled player immediately and works with imported local music without a server. A server URL is optional and is used only for shared listening rooms. The bundled player has no general native filesystem or shell permissions.

## Self-Hosted Server

The server is optional; the desktop app works without it. The app connects directly to Spotify using the Client ID entered in Settings; the room server is only needed for shared listening.

Create a Spotify app in the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard), then register `http://127.0.0.1:43821/callback` as a Redirect URI and enter its Client ID in Settings. No client secret is needed. Sign-in uses PKCE in the system browser, and the refresh token is stored in the operating system credential vault.

Room host control uses a random bearer token saved in the app's local storage so it survives restarts; the server stores only its hash. Rooms created before host-token protection must be recreated by their host.

## Local Development

From `svelte-frontend`, run `npm run dev`. The Windows launcher starts the local Qobuz API on `127.0.0.1:8000`, the audio/SpotiFLAC service on `127.0.0.1:8787`, and Vite on port `5173`. It does not start MongoDB or the room API; follow the server setup in [README.md](./README.md) if you need shared listening. For browser development, add the exact app URL you use (for example `http://127.0.0.1:5173/settings`) as a Spotify Redirect URI. Spotify catalog search through the audio service uses `SPOTIFY_CLIENT_ID` and `SPOTIFY_CLIENT_SECRET` from the repository `.env`; YouTube search does not require those credentials. If the system Python lacks the service dependencies, the launcher creates a virtual environment and installs both service requirements.

## Build the Installer

From `svelte-frontend`:

```powershell
npm install
npm run desktop:build
```

The NSIS installer is written under `svelte-frontend/src-tauri/target/release/bundle/nsis/`. WebView2 is not bundled, keeping the installer small; Windows downloads its bootstrapper during installation when the runtime is missing. The current build is unsigned; production distribution should use a Windows code-signing certificate.

## Room Playback

The host publishes the queue, current track ID, playback state, and playback clock through the room API; guests poll and synchronize playback. Local audio is uploaded only when the host adds it to the queue, then streamed by room members from the server. Uploaded audio is removed after seven days. Local files that are not queued remain on the device.