# Desktop Client

The desktop app is a native Dioxus Desktop application and works with imported local music without a server. A server URL is optional and is used only for shared listening rooms. Its initial window size is 1280x800 pixels, with a minimum size of 750x600 pixels.

## Self-Hosted Server

The server is optional; the desktop app works without it. The app connects directly to Spotify using its bundled public Client ID; users do not need to configure one. The room server is only needed for shared listening.

In the Spotify Developer Dashboard, register each exact Redirect URI that will be used:

- `http://127.0.0.1:43821/callback` for the desktop app.
- `http://127.0.0.1:8080/settings` for Dioxus web development.

The host, port, and path must match exactly. No client secret is used. Sign-in uses PKCE in the system browser, and the desktop refresh token is stored in the operating system credential vault.

Room host control uses a random bearer token saved in the app's local storage so it survives restarts; the server stores only its hash. Rooms created before host-token protection must be recreated by their host.

## Local Development

From the repository root, run `npm run dev` for the Dioxus web app or `npm run desktop:dev` for the native desktop app. The web launcher starts the local Qobuz API on `127.0.0.1:8000`, the audio/SpotiFLAC service on `127.0.0.1:8787`, and Dioxus on port `8080`. The desktop app starts its bundled audio service itself. Neither launcher starts MongoDB or the room API; follow the server setup in [README.md](./README.md) if you need shared listening. Register the Dioxus development Redirect URI listed above when using the web build. The audio service's separate Spotify catalog search uses `SPOTIFY_CLIENT_ID` and `SPOTIFY_CLIENT_SECRET` from the repository `.env`; these are not needed for the app's account-link feature. YouTube search does not require Spotify credentials. If the system Python lacks the service dependencies, the web launcher creates a virtual environment and installs both service requirements.

Firefox does not let a web app write into an arbitrary folder. It imports selected folders as snapshots stored in browser storage; add the folder again to include later changes. Browser downloads use Firefox's configured download location, which can be changed in Firefox Settings under General > Files and Applications. Use the Wavelength desktop app's native folder picker to choose an app-specific custom download folder. Large imported folders may exceed browser storage limits.

## Build for Windows

From the repository root:

```powershell
npm install
npm run desktop:build
```

The NSIS installer is written under `dist/desktop/`. WebView2 is provided by Windows and is not included in the installer. Production distribution should use a Windows code-signing certificate.

## Build for macOS

Install the Dioxus Desktop prerequisites, Python 3, Node.js, and Rust. From the repository root, run:

```sh
npm install
npm run desktop:build:unix
```

The script packages the local services for the host OS before running `dx bundle`. macOS bundles are written to `dist/desktop/`. Linux desktop packaging is not currently configured.

## Room Playback

The host publishes the queue, current track ID, playback state, and playback clock through the room API; guests poll and synchronize playback. Local audio is uploaded only when the host adds it to the queue, then streamed by room members from the server. Uploaded audio is removed after seven days. Local files that are not queued remain on the device.