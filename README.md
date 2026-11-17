# Wavelength

A local music player with Spotify and YouTube search, a bundled audio service, and an optional self-hosted server for listening rooms.

## Project layout

- `dioxus-frontend/` — Dioxus web and desktop UI.
- `scripts/` — Dioxus and local-service build/run tools.
- `audio-service/` — local audio search, preparation, and streaming service.
- `rust-backend/` — self-hosted room API and backend.
- `qobuz-rest-api/` — Qobuz service used by the local audio service; clone the repository with its submodules.

## Requirements

- Git
- Node.js and npm
- Python 3
- Rust stable and the platform prerequisites for Dioxus Desktop builds
- Dioxus CLI (`cargo install dioxus-cli --version 0.7.10`)
- Docker Engine with BuildKit to run the self-hosted server

## Run the app locally on Windows

Clone the repository with its Qobuz submodule, then install the project dependencies:

```powershell
git clone --recurse-submodules https://github.com/zespook/wavelength-rust.git
Set-Location wavelength-rust
Copy-Item .env.example .env
npm install
```

Start the web app and its local Qobuz/audio services:

```powershell
npm run dev
```

The Dioxus web app is available at `http://127.0.0.1:8080`. The launcher starts the local Qobuz API on port `8000` and audio service on port `8787`; it installs service Python dependencies into a virtual environment when needed. Spotify account search works after direct Spotify sign-in. The audio service's separate catalog search uses Spotify client credentials in the repository `.env`; those credentials are not needed to link an account. YouTube search does not require Spotify credentials.

To run the native desktop app:

```powershell
npm run desktop:dev
```

To build the Windows installer:

```powershell
npm run desktop:build
```

The Windows installer is written to `dist/desktop/`. See [DESKTOP.md](./DESKTOP.md) for additional desktop and deployment details.

On macOS or Linux, run `npm run desktop:build:unix` to build the host platform's desktop bundle.
Local desktop builds also write a matching compiled-services ZIP under `dist/release-assets/`; keep it there until the local installer has finished installing the services.

## Connect Spotify directly

Spotify can be connected without the self-hosted room server or per-user setup. Wavelength includes its public Spotify Client ID and uses PKCE; no client secret is needed or included. The Spotify app owner must register the exact Redirect URIs used by the app: `http://127.0.0.1:43821/callback` for desktop and `http://127.0.0.1:8080/settings` for Dioxus development. A web deployment must also register its exact `/settings` URL. Sign-in opens Spotify in the system browser, and account data is loaded directly from Spotify.

## Release versions

Push a semantic version tag such as `v1.2.3` to build a desktop release. The tag must match the version in `dioxus-frontend/Cargo.toml`. Each release publishes small main installers and a separate, platform-specific ZIP containing the compiled local services. Windows and macOS installers download and verify that ZIP during installation. Linux `.deb` and `.rpm` packages do the same from their install hooks; an AppImage downloads it on first launch. Users do not need Python or Node.js installed. The AppImage, `.deb`, and `.rpm` are all published for Linux.

The Linux AppImage is intended to be the broadest single-file Linux option, but no one package works on literally every distribution: host graphics/WebKit libraries and Linux ABI compatibility still apply. An internet connection is required when the compiled services are first installed.

## Run the self-hosted server with Docker

Dockerfiles and Compose manifests are intentionally not included. The following Bash instructions build the Rust room API using an inline Dockerfile and run it with MongoDB. From the repository root, copy `.env.example` to `.env`, then set `ROOM_SERVER_PASSCODE` to a unique value of at least 16 characters. Wavelength uses direct Spotify OAuth with its bundled public Client ID; the room server is not involved in Spotify sign-in.

Build the backend image:

```bash
docker build --tag wavelength-rooms-api --file - ./rust-backend <<'DOCKERFILE'
FROM rust:bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --bin wavelength-backend

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/wavelength-backend /usr/local/bin/wavelength-backend
EXPOSE 5000
CMD ["wavelength-backend"]
DOCKERFILE
```

Create a private Docker network and persistent volumes, then start MongoDB:

```bash
docker network create wavelength
docker volume create wavelength_rooms_data
docker volume create wavelength_rooms_audio

docker run -d \
  --name wavelength-mongodb \
  --restart unless-stopped \
  --network wavelength \
  --network-alias mongodb \
  --volume wavelength_rooms_data:/data/db \
  --health-cmd 'mongosh --quiet --eval "db.runCommand({ ping: 1 }).ok"' \
  --health-interval 10s \
  --health-timeout 5s \
  --health-retries 10 \
  mongo:6
```

Wait for MongoDB to report healthy:

```bash
until [ "$(docker inspect --format='{{.State.Health.Status}}' wavelength-mongodb)" = healthy ]; do sleep 2; done
```

Read the configured host port from `.env` (defaults to `5050`) and start the Rust room API:

```bash
ROOM_SERVER_PORT=$(awk -F= '$1 == "ROOM_SERVER_PORT" { print $2; exit }' .env)
ROOM_SERVER_PORT=${ROOM_SERVER_PORT:-5050}

docker run -d \
  --name wavelength-rooms-api \
  --restart unless-stopped \
  --network wavelength \
  --publish "${ROOM_SERVER_PORT}:5000" \
  --env-file .env \
  --env PORT=5000 \
  --env MONGODB_URI=mongodb://mongodb:27017/wavelength \
  --env MUSIC_ROOT=/music \
  --mount source=wavelength_rooms_audio,target=/music \
  wavelength-rooms-api
```

The server API is available at `http://localhost:5050`; it does not host the app UI. Enter that address in the app's Settings for same-device use, or use an HTTPS reverse proxy for remote clients. Room traffic uses the versioned `/api/v2` REST API and a direct JSON WebSocket at `/api/v2/rooms/{code}/socket`. Route both paths to the same service and pass WebSocket upgrades. The local Dioxus origins on port `8080` are allowed by default. For a deployed web app, set `FRONTEND_URL` in the room server's `.env` to its exact HTTPS origin without a path, then restart the server.

### Room API v2

Authenticate to a room server with `POST /api/v2/session` and `{"passcode":"..."}`; the response contains a short-lived `token`. Send that token in `x-room-access-token` on room API requests. Create a room with `POST /api/v2/rooms` and `{"host":"..."}`; join with `POST /api/v2/rooms/join` and `{"code":"AB12","username":"..."}`. Both responses include `code` and `party`; creation also returns `hostToken`.

Use `GET`, `PUT`, or `DELETE /api/v2/rooms/{code}` to read state, update host-owned queue/playback/playlist/permission state, or end a room. Host writes and local-audio uploads require `Authorization: Bearer {hostToken}`. Guests leave with `POST /api/v2/rooms/{code}/leave` and `{"username":"..."}`. Host-local audio uses `POST /api/v2/rooms/{code}/audio`; clients obtain a temporary stream URL from `GET /api/v2/rooms/{code}/audio/{audioId}/access`.

Before opening a WebSocket, clients request a 60-second, one-use ticket from `POST /api/v2/rooms/{code}/socket-ticket` with `{"username":"..."}` and the normal room access/host authorization headers. Connect to `/api/v2/rooms/{code}/socket?ticket=...`; access and host tokens are not placed in the WebSocket URL. Clients send JSON `{"event":"...","data":{...}}` messages for `room-command`, `playback-control`, and `playback-progress`. Server events use the same envelope with `room-state`, `room-command`, playback events, or `room-closed`. Room state is pushed to guests over this connection; clients do not poll for guest state. Rooms close five minutes after the last live room socket disconnects. Saved client sessions continue to restore through the same v2 API.

The server recognizes `CF-Connecting-IP`, `True-Client-IP`, `Fastly-Client-IP`, `X-Real-IP`, `X-Client-IP`, `X-Forwarded-For`, and the RFC `Forwarded` header. It ignores these headers unless the direct peer is listed in `ROOM_SERVER_TRUSTED_PROXIES`. Set every proxy hop's IP when relying on a forwarded chain, and configure proxies to set or append client-IP headers correctly. Never trust arbitrary client-supplied headers.

For Cloudflare Tunnel, point the public hostname at the room API service (for example, `http://wavelength-rooms-api:5000`) without a path restriction and enable **WebSockets** in Cloudflare's **Network** settings. Set `ROOM_SERVER_TRUSTED_PROXIES` to the tunnel container's IP as seen by the API container; the API then reads `CF-Connecting-IP` only from that trusted proxy.

View server logs with `docker logs -f wavelength-rooms-api`. To stop the containers, run `docker stop wavelength-rooms-api wavelength-mongodb`. The persistent volumes remain so room data is preserved.
