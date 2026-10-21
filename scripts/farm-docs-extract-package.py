"""Read a frozen npm archive, validate members, and extract to a new external directory."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import tarfile

MAX_ARCHIVE = 256 * 1024 * 1024
MAX_EXPANDED = 1024 * 1024 * 1024
MAX_MEMBERS = 50_000


def _member(item):
    name = item.name.rstrip("/") if item.isdir() else item.name
    parts = name.split("/")
    if not parts or parts[0] != "package":
        raise ValueError("Archive member must use npm package/ root")
    reserved = {"CON", "PRN", "AUX", "NUL", *[f"COM{i}" for i in range(1, 10)], *[f"LPT{i}" for i in range(1, 10)]}
    for part in parts:
        if (part in ("", ".", "..") or part.rstrip(" .") != part
                or any(char in part for char in '\\:<>"|?*')
                or any(ord(char) < 32 for char in part)
                or part.split(".", 1)[0].upper() in reserved):
            raise ValueError(f"Invalid archive member path: {name}")
    if not (item.isfile() or item.isdir()):
        raise ValueError(f"Unsupported archive member type/link: {name}")
    if len(parts) == 1:
        if not item.isdir():
            raise ValueError("npm package root must be a directory")
        return ""
    return "/".join(parts[1:])


def extract_package(archive_path, expected_sha256, destination, repository):
    archive_path, destination, repository = Path(archive_path).resolve(), Path(destination).resolve(), Path(repository).resolve(strict=True)
    if destination == repository or repository in destination.parents or destination in repository.parents:
        raise ValueError("Extraction destination and repository must be separate")
    if destination.exists():
        raise FileExistsError(f"Extraction destination already exists: {destination}")
    if not re.fullmatch(r"[a-fA-F0-9]{64}", expected_sha256):
        raise ValueError("Expected archive digest must be SHA256")
    if archive_path.stat().st_size > MAX_ARCHIVE:
        raise ValueError("Compressed archive exceeds extraction limit")
    with archive_path.open("rb") as source:
        data = source.read(MAX_ARCHIVE + 1)
    if len(data) > MAX_ARCHIVE:
        raise ValueError("Compressed archive exceeds extraction limit")
    digest = hashlib.sha256(data).hexdigest()
    if digest != expected_sha256.lower():
        raise ValueError("Archive digest differs from pack receipt")
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        entries, seen, prefixes, regular = [], set(), {}, set()
        expanded = 0
        for index, item in enumerate(archive):
            if index >= MAX_MEMBERS:
                raise ValueError("Archive member count exceeds extraction limit")
            name = _member(item)
            if name in seen:
                raise ValueError(f"Duplicate archive member: {name}")
            seen.add(name)
            if not name:
                continue
            parts = name.split("/")
            for end in range(1, len(parts) + 1):
                prefix = "/".join(parts[:end])
                previous = prefixes.setdefault(prefix.casefold(), prefix)
                if previous != prefix:
                    raise ValueError(f"Archive path case collision: {name}")
            if item.isfile():
                regular.add(name)
                expanded += item.size
                if item.size < 0 or expanded > MAX_EXPANDED:
                    raise ValueError("Expanded archive exceeds extraction limit")
            entries.append((name, item))
        for name, item in entries:
            parts = name.split("/")
            if any("/".join(parts[:end]) in regular for end in range(1, len(parts))):
                raise ValueError(f"Archive file/directory conflict: {name}")
        manifest_item = next((item for name, item in entries if name == "package.json" and item.isfile()), None)
        if manifest_item is None or manifest_item.size > 1024 * 1024:
            raise ValueError("Missing or oversized package.json")
        with archive.extractfile(manifest_item) as source:
            manifest = json.loads(source.read().decode("utf-8"))
        if not isinstance(manifest, dict) or not all(isinstance(manifest.get(key), str) and manifest[key] for key in ("name", "version")):
            raise ValueError("Invalid package.json identity")
        destination.mkdir(parents=True, exist_ok=False)
        files = []
        for name, item in entries:
            target = destination.joinpath(*name.split("/"))
            if item.isdir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            file_hash, size = hashlib.sha256(), 0
            with archive.extractfile(item) as source, target.open("xb") as output:
                while chunk := source.read(1024 * 1024):
                    output.write(chunk)
                    file_hash.update(chunk)
                    size += len(chunk)
            if size != item.size:
                raise ValueError(f"Extracted member size differs: {name}")
            files.append({"path": name, "bytes": size, "sha256": file_hash.hexdigest()})
    return {"archive": str(archive_path), "sha256": digest, "destination": str(destination), "manifest": manifest,
            "files": sorted(files, key=lambda row: row["path"]), "scope": "Validated npm archive extraction only; no package execution, install, ABI or build acceptance."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ("archive", "sha256", "destination", "repository", "receipt"):
        parser.add_argument("--" + option, required=True)
    options = parser.parse_args()
    # Reserve the new receipt before extraction; failures retain a truthful failure receipt.
    with Path(options.receipt).open("x", encoding="utf-8") as receipt:
        try:
            report = extract_package(options.archive, options.sha256, options.destination, options.repository)
        except Exception as error:
            json.dump({"success": False, "error": str(error)}, receipt, ensure_ascii=False, indent=2)
            raise
        json.dump({"success": True, **report}, receipt, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
