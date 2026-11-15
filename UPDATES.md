# Desktop updates

Desktop releases are published from GitHub tags matching `v*`. The workflow builds a Windows installer and macOS DMGs, signs the update packages, and publishes a `latest.json` feed. Native Dioxus clients verify package signatures before installing.

## Publish a release

1. Update the `version` in `dioxus-frontend/Cargo.toml` (for example, `0.1.9` to `0.2.0`).
2. Commit and push the changes to the GitHub repository.
3. Tag that commit with the matching version and push the tag:

	```powershell
	git tag v0.2.0
	git push origin v0.2.0
	```

The `Release Wavelength Desktop` workflow runs on `v*` tags and requires the tag version to match `dioxus-frontend/Cargo.toml`. It packages the local services for each OS, bundles the native Dioxus app, signs each install/update package, and publishes the assets and `latest.json`. Keep the signing key pair stable across releases; installed clients use the public key embedded in the app to verify updates.

## Install an update

Desktop clients check for updates 10 seconds after launch and every six hours while running. When a signed release is found, Wavelength shows an in-app notification with an **Install update** action. Installation rechecks the release version, verifies and downloads the signed package, installs it, and relaunches the app. A manual **Check now** action is also available in Settings. Offline clients discover updates on their next launch or scheduled check.

The app must embed the updater public key that matches the release signing key. The app points to `https://github.com/zespook/wavelength-rust/releases/latest/download/latest.json`; manually install a signed release first if an existing build predates updater support or uses a different key. Later signed releases can then be installed from the app.

## One-time signing setup

1. Generate a new Minisign key pair for this repository with `minisign -G`. Update `UPDATE_PUBLIC_KEY` in `dioxus-frontend/src/desktop.rs` to the new public key before releasing. Do not commit the private key.
2. Add the private key contents as the GitHub Actions secret `UPDATE_SIGNING_PRIVATE_KEY`.
3. If the key is password-protected, add its password as `UPDATE_SIGNING_PRIVATE_KEY_PASSWORD`. Leave that secret unset for an unencrypted key.
4. Update the version in `dioxus-frontend/Cargo.toml`, then push a matching tag such as `v0.2.0`.

The private key must never be committed. Existing installs that trust the previous public key will not accept releases signed with the new key; distribute a build containing the new public key manually first. The release workflow derives update asset URLs from the repository running it. macOS packages are built for both Intel and Apple Silicon; Windows packages are built for x86-64.
