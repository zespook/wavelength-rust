#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "$0")/.." && pwd)"
AUDIO_DIR="$ROOT_DIR/audio-service"
QOBUZ_DIR="$ROOT_DIR/qobuz-rest-api"
RESOURCE_DIR="$ROOT_DIR/src-tauri/resources/services"
BUILD_ROOT="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/wavelength-desktop-service-build"
BUILD_VENV="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/wavelength-service-build-venv"
VENV_PYTHON="$BUILD_VENV/bin/python"

if [[ ! -x "$VENV_PYTHON" ]]; then
	python3 -m venv "$BUILD_VENV"
fi

"$VENV_PYTHON" -m pip install --disable-pip-version-check \
	-r "$AUDIO_DIR/requirements.txt" \
	-r "$QOBUZ_DIR/requirements.txt" \
	pyinstaller==6.12.0

rm -rf "$BUILD_ROOT" "$RESOURCE_DIR"
SERVICE_DIST="$BUILD_ROOT/dist"
SERVICE_WORK="$BUILD_ROOT/work"
SERVICE_SPEC="$BUILD_ROOT/spec"
API_RESOURCE_DIR="$RESOURCE_DIR/api"
RUNTIME_RESOURCE_DIR="$RESOURCE_DIR/runtime"
mkdir -p "$SERVICE_DIST" "$SERVICE_WORK" "$SERVICE_SPEC" \
	"$API_RESOURCE_DIR" "$RUNTIME_RESOURCE_DIR"

"$VENV_PYTHON" -m PyInstaller \
	--noconfirm --clean --onedir \
	--name WavelengthLocalService \
	--distpath "$SERVICE_DIST" \
	--workpath "$SERVICE_WORK" \
	--specpath "$SERVICE_SPEC" \
	--paths "$AUDIO_DIR" \
	--paths "$QOBUZ_DIR" \
	--add-data "$AUDIO_DIR/main.py:audio" \
	--add-data "$AUDIO_DIR/playlist.py:audio" \
	--add-data "$QOBUZ_DIR/main.py:qobuz" \
	--hidden-import playlist \
	--collect-all fastapi \
	--collect-all uvicorn \
	--collect-all starlette \
	--collect-all pydantic \
	--collect-all httpx \
	--collect-all spotipy \
	--collect-all SpotiFLAC \
	--collect-all yt_dlp \
	--collect-all packaging \
	--collect-all mutagen \
	--collect-all certifi \
	--copy-metadata SpotiFLAC \
	"$AUDIO_DIR/desktop_service.py"

cp -R "$SERVICE_DIST/WavelengthLocalService/." "$API_RESOURCE_DIR/"
NODE_BINARY="$(command -v node)"
cp -pL "$NODE_BINARY" "$RUNTIME_RESOURCE_DIR/node"
chmod 755 "$RUNTIME_RESOURCE_DIR/node" \
	"$API_RESOURCE_DIR/WavelengthLocalService"

if [[ ! -x "$API_RESOURCE_DIR/WavelengthLocalService" || ! -x "$RUNTIME_RESOURCE_DIR/node" ]]; then
	echo "The bundled local service or Node.js runtime is missing." >&2
	exit 1
fi

echo "Combined local audio/Qobuz service and Node.js runtime are ready for Tauri bundling."