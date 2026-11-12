# Desktop Client

The Windows app is a small Tauri client using the system WebView2 runtime. It opens the bundled player immediately and works with imported local music without a server. A server URL is optional and is used only for shared listening rooms. The bundled player has no general native filesystem or shell permissions.

## Self-Hosted Server

The server is optional; the desktop app works without it. The app connects directly to Spotify using its bundled public Client ID; users do not need to configure one. The room server is only needed for shared listening.

In the Spotify Developer Dashboard, register each exact Redirect URI that will be used:

- `http://127.0.0.1:43821/callback` for the desktop app.
- `http://127.0.0.1:5173/settings` for Vite development.
- `http://127.0.0.1:4173/settings` for Vite preview.

The host, port, and path must match exactly. No client secret is used. Sign-in uses PKCE in the system browser, and the desktop refresh token is stored in the operating system credential vault.

Room host control uses a random bearer token saved in the app's local storage so it survives restarts; the server stores only its hash. Rooms created before host-token protection must be recreated by their host.

## Local Development

From `svelte-frontend`, run `npm run dev`. The Windows launcher starts the local Qobuz API on `127.0.0.1:8000`, the audio/SpotiFLAC service on `127.0.0.1:8787`, and Vite on port `5173`. It does not start MongoDB or the room API; follow the server setup in [README.md](./README.md) if you need shared listening. Register the Vite development and preview Redirect URIs listed above if using those browser builds. The audio service's separate Spotify catalog search uses `SPOTIFY_CLIENT_ID` and `SPOTIFY_CLIENT_SECRET` from the repository `.env`; these are not needed for the app's account-link feature. YouTube search does not require Spotify credentials. If the system Python lacks the service dependencies, the launcher creates a virtual environment and installs both service requirements.

Firefox does not let a web app write into an arbitrary folder. It imports selected folders as snapshots stored in browser storage; add the folder again to include later changes. Browser downloads use Firefox's configured download location, which can be changed in Firefox Settings under General > Files and Applications. Use the Wavelength desktop app's native folder picker to choose an app-specific custom download folder. Large imported folders may exceed browser storage limits.

## Build the Installer

From `svelte-frontend`:

```powershell
npm install
npm run desktop:build
```

The NSIS installer is written under `svelte-frontend/src-tauri/target/release/bundle/nsis/`. WebView2 is not bundled, keeping the installer small; Windows downloads its bootstrapper during installation when the runtime is missing. The current build is unsigned; production distribution should use a Windows code-signing certificate.

## Room Playback

The host publishes the queue, current track ID, playback state, and playback clock through the room API; guests poll and synchronize playback. Local audio is uploaded only when the host adds it to the queue, then streamed by room members from the server. Uploaded audio is removed after seven days. Local files that are not queued remain on the device.