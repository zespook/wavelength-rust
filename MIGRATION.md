# Dioxus application

The Dioxus frontend is the only supported Wavelength client. Dioxus Desktop provides the native window and integrates local service management, folder selection, downloads, and updater support.

## Project layout

- `dioxus-frontend/` contains the Rust/Dioxus UI and its static assets.
- `audio-service/` and `qobuz-rest-api/` provide local search, playback, and download services.
- `rust-backend/` provides the optional self-hosted room API.

## Development

Install Node.js, Rust, Python, Dioxus CLI 0.7.10, and the platform prerequisites. From the repository root, `npm run dev` starts Dioxus and the local audio services at `http://127.0.0.1:8080`; `npm run desktop:dev` launches the native Dioxus application.

Build the Dioxus web assets with `npm run build:dioxus`. Build a Windows installer with `npm run desktop:build`, or a macOS app with `npm run desktop:build:unix`.

The room API is optional. Shared listening requires the Rust backend and MongoDB; see the Docker setup in [README.md](./README.md).

## Platform notes

The web and desktop clients share the same layout; there is no separate mobile-optimized client. Spotify sign-in, room synchronization, and shared playback require the relevant Spotify app configuration and/or a running room server for end-to-end testing.
