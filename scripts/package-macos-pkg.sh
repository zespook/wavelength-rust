#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
	echo "Usage: package-macos-pkg.sh <version> <app-archive> <output-pkg>" >&2
	exit 2
fi

VERSION="$1"
APP_ARCHIVE="$2"
OUTPUT_PKG="$3"
ROOT_DIR="$(cd -- "$(dirname -- "$0")/.." && pwd)"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/wavelength-pkg.XXXXXX")"
trap 'rm -rf "$WORK_DIR"' EXIT

UNPACK_DIR="$WORK_DIR/unpacked"
ROOT="$WORK_DIR/root"
SCRIPTS="$WORK_DIR/scripts"
mkdir -p "$UNPACK_DIR" "$ROOT/Applications" "$SCRIPTS"
tar -xzf "$APP_ARCHIVE" -C "$UNPACK_DIR"

APP_BUNDLE="$(find "$UNPACK_DIR" -maxdepth 3 -type d -name '*.app' -print -quit)"
if [[ -z "$APP_BUNDLE" ]]; then
	echo "The Dioxus update archive does not contain a macOS app bundle." >&2
	exit 1
fi

ditto "$APP_BUNDLE" "$ROOT/Applications/Wavelength.app"
cp "$ROOT_DIR/scripts/macos-pkg-postinstall" "$SCRIPTS/postinstall"
chmod 755 "$SCRIPTS/postinstall"
mkdir -p "$(dirname "$OUTPUT_PKG")"

pkgbuild \
	--root "$ROOT" \
	--scripts "$SCRIPTS" \
	--identifier "com.wavelength.desktop.installer" \
	--version "$VERSION" \
	--install-location "/" \
	"$OUTPUT_PKG"
