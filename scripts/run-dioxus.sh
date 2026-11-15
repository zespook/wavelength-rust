#!/usr/bin/env bash
set -euo pipefail

ACTION="${1:-dev}"
ROOT_DIR="$(cd -- "$(dirname -- "$0")/.." && pwd)"
FRONTEND_DIR="$ROOT_DIR/dioxus-frontend"

if [[ "$ACTION" == "build" ]]; then
	"$ROOT_DIR/scripts/package-services-macos.sh"
	mkdir -p "$ROOT_DIR/dist/desktop"
	cd "$FRONTEND_DIR"
	dx bundle --platform desktop --release --no-default-features --features desktop \
		--package-types macos \
		--package-types dmg \
		--package-types updater \
		--out-dir "$ROOT_DIR/dist/desktop"
elif [[ "$ACTION" == "dev" ]]; then
	"$ROOT_DIR/scripts/package-services-macos.sh"
	cd "$FRONTEND_DIR"
	dx serve --platform desktop --no-default-features --features desktop
else
	echo "Usage: $0 [dev|build]" >&2
	exit 2
fi
