# Rust + Svelte migration

The production stack now uses `svelte-frontend` for the application shell and `rust-backend` as the public backend entry point.

## Current boundary

- `rust-backend` owns the public port, MongoDB party APIs, Spotify OAuth/library APIs, request validation, audio proxying, lyrics lookup, and upstream failure handling.
- Local files are mounted read-only at `/music` and exposed through `/api/library`; set `MUSIC_LIBRARY_PATH` to the host folder in `.env`.
- Users can also choose a local folder directly in a supported browser; those files stay in the browser and can be played or downloaded without server mounting.
- Search combines the browser-selected/server library with Spotify track, album, and playlist results. SpotiFLAC uses Qobuz, Tidal, Amazon, and Deezer as download fallbacks; those providers are not independent metadata search indexes.
- Admin accounts are stored in MongoDB and managed through `/api/admin/users`; each account carries a `canDownload` permission.
- `audio-service`, `qobuz-api`, and MongoDB are unchanged.
- The Svelte UI uses a Sonara-inspired library-first layout while keeping party creation/joining visible beside solo playback.

The old Express service and React client have been removed. Encrypted Qobuz credential storage, the Svelte client's live party event protocol, and enforcement of download permissions on a dedicated authenticated download endpoint still need to be completed. The current browser download action works for local files and streamed track results when the user has access to the audio service.

## Local development

```powershell
Push-Location svelte-frontend
npm install
npm run dev
Pop-Location
```

The Rust room API requires MongoDB. See the Docker server setup in `README.md`; for local runs, set `MONGODB_URI` to the MongoDB address.
