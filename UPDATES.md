# Desktop updates

Desktop releases are published from GitHub tags matching `v*`. The Windows workflow builds a signed NSIS installer and `latest.json`; installed Tauri clients poll that feed and verify signatures before installing.

## Publish a release

1. Update `version` in `src-tauri/tauri.conf.json` (for example, `0.1.0` to `0.2.0`).
2. Commit and push the changes to the GitHub repository.
3. Tag that commit with the matching version and push the tag:

	```powershell
	git tag v0.2.0
	git push origin v0.2.0
	```

The `Release Wavelength Desktop` workflow runs on `v*` tags. It packages the local audio services, configures the signed updater feed, builds the NSIS installer, and publishes the installer and `latest.json` to a GitHub Release. Keep the signing key pair stable across releases; installed clients use the embedded public key to verify updates.

## Install an update

Desktop clients check for updates 10 seconds after launch and every six hours while running. When a signed release is found, Wavelength shows an in-app notification with an **Install update** action. Installation verifies and downloads the update, installs it, and relaunches the app. A manual **Check now** action is also available in Settings. Offline clients discover updates on their next launch or scheduled check.

The current locally built installer is unsigned and has no updater feed configured, so it cannot self-update; users must install the first signed GitHub Release manually. Clients installed from that signed release can receive later updates automatically.

## One-time signing setup

1. Generate a Tauri updater key pair with `npx tauri signer generate -w <private-key-path>` from the repository root.
2. Add the generated private key as the GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY`.
3. If the key has a password, add `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as a GitHub Actions secret. Leave it empty for an unencrypted key.
4. Add the matching public key as the repository Actions variable `TAURI_UPDATER_PUBLIC_KEY`.
5. Push a tag such as `v0.2.0` after updating the app version in `src-tauri/tauri.conf.json`.

The private key must never be committed. The release workflow derives the update endpoint from the GitHub repository that runs it. This checkout has no Git remote, so the workflow must be committed to the intended GitHub repository before a real remote update can be published and tested.
