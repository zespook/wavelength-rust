#!/usr/bin/env python3
import hashlib
import os
import re
import shutil
import stat
import sys
import zipfile
from pathlib import Path

MAX_ARCHIVE_BYTES = 2 * 1024 * 1024 * 1024
MAX_UNPACKED_BYTES = 4 * 1024 * 1024 * 1024


def fail(message):
    raise SystemExit(message)


if len(sys.argv) != 4:
    fail("Usage: package-compiled-services.py <services-dir> <version> <output.zip>")

services_dir = Path(sys.argv[1]).resolve(strict=True)
version = sys.argv[2]
output_path = Path(sys.argv[3])
if not re.fullmatch(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)", version):
    fail("The service package version must be semantic MAJOR.MINOR.PATCH.")

executable_name = "WavelengthLocalService.exe" if os.name == "nt" else "WavelengthLocalService"
node_name = "node.exe" if os.name == "nt" else "node"
for required in (
    services_dir / "api" / executable_name,
    services_dir / "runtime" / node_name,
):
    if not required.is_file():
        fail(f"Required compiled service file is missing: {required}")

root = services_dir.resolve(strict=True)
output_path.parent.mkdir(parents=True, exist_ok=True)
if output_path.exists():
    output_path.unlink()


def add_tree(archive, source, archive_name, ancestors):
    resolved = source.resolve(strict=True)
    if not resolved.is_relative_to(root):
        fail(f"Service package entry points outside the service directory: {source}")
    if resolved in ancestors:
        fail(f"Service package contains a directory link cycle: {source}")
    if source.is_dir():
        next_ancestors = ancestors | {resolved}
        for child in sorted(source.iterdir(), key=lambda item: item.name):
            add_tree(archive, child, f"{archive_name}/{child.name}", next_ancestors)
        return
    if not source.is_file():
        fail(f"Unsupported service package entry: {source}")

    mode = stat.S_IMODE(source.stat().st_mode)
    if os.name == "nt":
        mode = 0o644
    info = zipfile.ZipInfo(archive_name, date_time=(1980, 1, 1, 0, 0, 0))
    info.create_system = 3
    info.compress_type = zipfile.ZIP_DEFLATED
    info.external_attr = (stat.S_IFREG | mode) << 16
    with resolved.open("rb") as source_file, archive.open(info, "w") as archive_file:
        shutil.copyfileobj(source_file, archive_file, length=1024 * 1024)


with zipfile.ZipFile(
    output_path,
    "w",
    compression=zipfile.ZIP_DEFLATED,
    compresslevel=6,
    allowZip64=True,
) as archive:
    for folder in ("api", "runtime"):
        add_tree(archive, services_dir / folder, folder, set())
    archive.writestr(
        ".wavelength-service-version",
        version + "\n",
        compress_type=zipfile.ZIP_DEFLATED,
    )

with zipfile.ZipFile(output_path) as archive:
    bad_entry = archive.testzip()
    if bad_entry is not None:
        fail(f"Compiled service archive failed its CRC check: {bad_entry}")
    unpacked_size = sum(entry.file_size for entry in archive.infolist())
    if unpacked_size > MAX_UNPACKED_BYTES:
        fail("The compiled services exceed the allowed extracted size.")

if output_path.stat().st_size > MAX_ARCHIVE_BYTES:
    fail("The compiled services exceed the allowed archive size.")

hasher = hashlib.sha256()
with output_path.open("rb") as archive_file:
    for chunk in iter(lambda: archive_file.read(1024 * 1024), b""):
        hasher.update(chunk)
digest = hasher.hexdigest()
print(f"Created {output_path} ({output_path.stat().st_size} bytes)")
print(f"sha256={digest}")
