#!/usr/bin/env bash
set -euo pipefail

ACTION="${1:-dev}"
ROOT_DIR="$(cd -- "$(dirname -- "$0")/.." && pwd)"
FRONTEND_DIR="$ROOT_DIR/dioxus-frontend"

if [[ "$ACTION" == "build" ]]; then
	"$ROOT_DIR/scripts/package-services-unix.sh"
	version="$(sed -nE 's/^version = "([^"]+)"$/\1/p' "$FRONTEND_DIR/Cargo.toml" | head -n 1)"
	[[ -n "$version" ]] || {
		echo "Could not read the app version from Cargo.toml." >&2
		exit 1
	}
	case "$(uname -s)-$(uname -m)" in
		Darwin-x86_64) platform="darwin-x86_64" ;;
		Darwin-arm64|Darwin-aarch64) platform="darwin-aarch64" ;;
		Linux-x86_64) platform="linux-x86_64" ;;
		*) echo "Compiled service packages are not available for this platform." >&2; exit 2 ;;
	esac
	service_archive="$ROOT_DIR/dist/release-assets/Wavelength_${version}_services_${platform}.zip"
	package_output="$(python3 "$ROOT_DIR/scripts/package-compiled-services.py" \
		"$FRONTEND_DIR/resources/services" "$version" "$service_archive")"
	printf '%s\n' "$package_output"
	service_hash="$(printf '%s\n' "$package_output" | sed -n 's/^sha256=//p')"
	[[ "$service_hash" =~ ^[[:xdigit:]]{64}$ ]] || {
		echo "Could not read the compiled service package checksum." >&2
		exit 1
	}
	export WAVELENGTH_SERVICES_SHA256="$service_hash"
	export WAVELENGTH_SERVICES_ARCHIVE="$service_archive"
	mkdir -p "$ROOT_DIR/dist/desktop"
	cd "$FRONTEND_DIR"
	if [[ "$(uname -s)" == "Darwin" ]]; then
		dx bundle --platform desktop --release --no-default-features --features desktop \
			--package-types macos \
			--package-types dmg \
			--package-types updater \
			--out-dir "$ROOT_DIR/dist/desktop"
	elif [[ "$(uname -s)" == "Linux" ]]; then
		dx bundle --platform desktop --release --no-default-features --features desktop \
			--package-types appimage \
			--package-types deb \
			--package-types rpm \
			--out-dir "$ROOT_DIR/dist/desktop"
	else
		echo "Desktop builds are supported on macOS and Linux." >&2
		exit 2
	fi
elif [[ "$ACTION" == "dev" ]]; then
	"$ROOT_DIR/scripts/package-services-unix.sh"
	cd "$FRONTEND_DIR"
	dx serve --platform desktop --no-default-features --features desktop
else
	echo "Usage: $0 [dev|build]" >&2
	exit 2
fi
