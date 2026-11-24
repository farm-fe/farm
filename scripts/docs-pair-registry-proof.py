"""Read-only adoption proof for the exact Farm Docs 2.0.0 pair.

The caller must independently authenticate the frozen producer evidence and the
isolated public-registry install receipt before calling this function.
"""
import base64
import binascii
import hashlib
import json
import re
import urllib.parse

VERSION = "2.0.0"
DOCS = "@farmfe/docs"
THEME = "@farmfe/docs-theme-default"
ORDER = (THEME, DOCS)
MAX_ARCHIVE_BYTES = 100 * 1024 * 1024
DEP_FIELDS = ("dependencies", "optionalDependencies", "peerDependencies", "peerDependenciesMeta")


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def identity(value, size):
    return isinstance(value, str) and re.fullmatch(r"[a-f0-9]{%d}" % size, value) is not None


def integrity(value):
    if not isinstance(value, str) or not value.startswith("sha512-"):
        return False
    try:
        digest = base64.b64decode(value[7:], validate=True)
    except (binascii.Error, ValueError):
        return False
    return len(digest) == 64 and base64.b64encode(digest).decode("ascii") == value[7:]


def registry_url(name, url):
    if not isinstance(url, str):
        return False
    filename = name.split("/", 1)[1]
    return url in {f"https://registry.npmjs.org/{name}/-/{filename}-{VERSION}.tgz",
                   f"https://registry.npmjs.org/{urllib.parse.quote(name, safe='@')}/-/{filename}-{VERSION}.tgz"}


def package_rows(receipt, *, kind, source_sha, source_tree, installed=False):
    require(isinstance(receipt, dict) and set(receipt) == {"schemaVersion", "kind", "sourceSHA", "sourceTree", "packages"},
            "Docs pair receipt fields invalid")
    require(type(receipt["schemaVersion"]) is int and receipt["schemaVersion"] == 1 and receipt["kind"] == kind
            and receipt["sourceSHA"] == source_sha and receipt["sourceTree"] == source_tree,
            "Docs pair receipt source differs")
    entries = receipt["packages"]
    require(isinstance(entries, list) and len(entries) == 2, "Expected exactly two Docs packages")
    fields = {"name", "version", "archiveSha256", "archiveBytes", "integrity"}
    if installed:
        fields.add("resolved")
    rows = {}
    for row in entries:
        require(isinstance(row, dict) and set(row) == fields, "Docs pair package fields invalid")
        name = row["name"]
        require(isinstance(name, str) and name in ORDER and name not in rows and row["version"] == VERSION,
                "Unexpected Docs package identity")
        require(identity(row["archiveSha256"], 64) and integrity(row["integrity"])
                and type(row["archiveBytes"]) is int and 0 < row["archiveBytes"] <= MAX_ARCHIVE_BYTES,
                "Invalid Docs package archive evidence")
        if installed:
            require(registry_url(name, row["resolved"]), "Install must resolve a public exact npm archive")
        rows[name] = row
    return rows


def metadata_for(name, manifest):
    require(isinstance(manifest, dict) and set(manifest) == set(DEP_FIELDS)
            and all(isinstance(manifest[field], dict) for field in DEP_FIELDS),
            "Public Docs dependency declaration fields invalid")
    counterpart, expected_field = (THEME, "dependencies") if name == DOCS else (DOCS, "peerDependencies")
    require(manifest[expected_field].get(counterpart) == VERSION
            and all(counterpart not in manifest[field] for field in DEP_FIELDS if field != expected_field),
            "Public Docs pair dependency is not exact and mandatory")
    return json.dumps(manifest, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def verify_pair(expected, installed, *, expected_source_sha, expected_source_tree, lookup):
    """Return retain-only evidence or raise; never publishes or writes tags.

    lookup is the bounded exact npm Registry.lookup with no registry writes.
    The installed receipt represents measured per-package archive identities
    and registry lock resolutions from an independently verified clean install.
    """
    require(identity(expected_source_sha, 40) and identity(expected_source_tree, 40),
            "Exact reviewed Docs commit and tree required")
    source = {"source_sha": expected_source_sha, "source_tree": expected_source_tree}
    expected_rows = package_rows(expected, kind="docs-frozen-pair-v1", **source)
    installed_rows = package_rows(installed, kind="docs-public-install-v1", installed=True, **source)
    for name in ORDER:
        row = installed_rows[name]
        require(all(row[key] == value for key, value in expected_rows[name].items()),
                "Installed Docs archive differs from frozen producer")

    # Verify two metadata endpoints before any tarball request, then measure
    # the actual bytes with the existing registry archive reader.
    observed = {}
    for verify_archive in (False, True):
        for name in ORDER:
            row = lookup({"kind": "npm", "name": name, "version": VERSION}, verify_archive=verify_archive)
            wanted = expected_rows[name]
            require(isinstance(row, dict) and row.get("status") == "published"
                    and row.get("name") == name and row.get("version") == VERSION,
                    "Exact Docs pair is not fully public")
            require(row.get("evidenceKind") == ("registry-archive" if verify_archive else "registry-metadata")
                    and row.get("integrity") == wanted["integrity"]
                    and registry_url(name, row.get("tarballURL"))
                    and row["tarballURL"] == installed_rows[name]["resolved"],
                    "Public Docs URL or SRI differs from frozen/installed identity")
            public_metadata = metadata_for(name, row.get("metadata"))
            if not verify_archive:
                require("archiveSha256" not in row and "archiveBytes" not in row,
                        "Metadata-only Docs lookup claims measured bytes")
                observed[name] = public_metadata
            else:
                require(row.get("archiveSha256") == wanted["archiveSha256"]
                        and type(row.get("archiveBytes")) is int
                        and row["archiveBytes"] == wanted["archiveBytes"]
                        and observed[name] == public_metadata,
                        "Public Docs archive bytes or dependency metadata changed")

    return {
        "schemaVersion": 1, "kind": "docs-public-retain-only-v1",
        "sourceSHA": expected_source_sha, "sourceTree": expected_source_tree,
        "packages": [{**expected_rows[name], "resolved": installed_rows[name]["resolved"],
                      "metadataSha256": hashlib.sha256(observed[name].encode()).hexdigest()} for name in ORDER],
    }
