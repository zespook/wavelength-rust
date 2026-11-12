# Wavelength

A local music player with Spotify and YouTube search, a bundled audio service, and an optional self-hosted server for listening rooms.

## Project layout

- `svelte-frontend/` — Svelte web app and Tauri desktop client.
- `audio-service/` — local audio search, preparation, and streaming service.
- `rust-backend/` — self-hosted room API and backend.
- `qobuz-rest-api/` — Qobuz service used by the local audio service; clone the repository with its submodules.

## Requirements

- Git
- Node.js and npm
- Python 3
- Rust stable and the Windows C++ build tools for Tauri development/builds
- Docker Engine with BuildKit to run the self-hosted server

## Run the app locally on Windows

Clone the repository with its Qobuz submodule, then install the frontend dependencies:

```powershell
git clone --recurse-submodules https://github.com/zespook/wavelength.git
Set-Location wavelength
Copy-Item .env.example .env
Set-Location svelte-frontend
npm install
```

Start the web app and its local Qobuz/audio services:

```powershell
npm run dev
```

The app is available at `http://127.0.0.1:5173`. The launcher starts the local Qobuz API on port `8000` and audio service on port `8787`; it installs service Python dependencies into a virtual environment when needed. Spotify account search works after direct Spotify sign-in. The audio service's catalog search still uses Spotify client credentials in the repository `.env`; YouTube search does not.

To run the desktop app instead, from `svelte-frontend` run:

```powershell
npm run desktop:dev
```

To build the Windows installer:

```powershell
npm run desktop:build
```

The installer is written to `svelte-frontend/src-tauri/target/release/bundle/nsis/`. See [DESKTOP.md](./DESKTOP.md) for additional desktop and deployment details.

## Connect Spotify directly

Spotify can be connected without the self-hosted room server. Create an app in the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard), then enter its Client ID in Wavelength Settings; a client secret is not needed. Add `http://127.0.0.1:43821/callback` to the app's Redirect URIs for the desktop client. A web deployment must also add its exact app URL as a Redirect URI. Sign-in uses PKCE in the system browser, and Spotify profile, saved tracks, playlists, and search results are loaded directly from Spotify.

## Release versions

Push a semantic version tag such as `v1.2.3` to build a desktop release. The release workflow uses the tag (without its leading `v`) as the Tauri app version, so installer and updater asset names use the tagged version automatically. GitHub release titles include the full tag.

## Run the self-hosted server with Docker

Dockerfiles and Compose manifests are intentionally not included. The following Bash instructions build the Rust room API using an inline Dockerfile and run it with MongoDB. From the repository root, copy `.env.example` to `.env`, then set `ROOM_SERVER_PASSCODE` to a unique value of at least 16 characters. Server-managed Spotify OAuth is optional; the Wavelength app can instead connect directly using the Client ID in Settings.

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

The server API is available at `http://localhost:5050`; it does not host the app UI. Enter that address in the app's Settings for same-device use, or use an HTTPS reverse proxy for remote clients. View server logs with `docker logs -f wavelength-rooms-api`. To stop the containers, run `docker stop wavelength-rooms-api wavelength-mongodb`. The persistent volumes remain so room data is preserved.
