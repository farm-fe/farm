import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("docs_extract", Path(__file__).with_name("farm-docs-extract-package.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ExtractionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="farm-docs-extract-")
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.repo = self.base / "repo"
        self.repo.mkdir()

    def archive(self, extras=(), include_manifest=True):
        data = io.BytesIO()
        with tarfile.open(fileobj=data, mode="w:gz") as archive:
            rows = []
            if include_manifest:
                rows.append(("package/package.json", json.dumps({"name": "@farmfe/docs", "version": "0.0.1"}).encode(), tarfile.REGTYPE))
            rows += [("package/dist/index.js", b"throw Error('never execute');", tarfile.REGTYPE), *extras]
            for name, body, kind in rows:
                item = tarfile.TarInfo(name)
                item.type = kind
                if kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
                    item.linkname = "../../outside"
                else:
                    item.size = len(body)
                archive.addfile(item, io.BytesIO(body) if kind == tarfile.REGTYPE else None)
        file = self.base / "fixture.tgz"
        file.write_bytes(data.getvalue())
        return file, hashlib.sha256(data.getvalue()).hexdigest()

    def extract(self, file, digest, destination=None):
        return module.extract_package(file, digest, destination or self.base / "extracted", self.repo)

    def test_real_tgz_manifest_and_files_are_bound_to_archive_digest(self):
        file, digest = self.archive([("package/assets", b"", tarfile.DIRTYPE), ("package/assets/mark.svg", b"<svg/>", tarfile.REGTYPE)])
        report = self.extract(file, digest)
        self.assertEqual(report["sha256"], digest)
        self.assertEqual(report["manifest"]["name"], "@farmfe/docs")
        self.assertEqual(len(report["files"]), 3)
        entry = next(row for row in report["files"] if row["path"] == "dist/index.js")
        self.assertEqual(entry["sha256"], hashlib.sha256(b"throw Error('never execute');").hexdigest())
        self.assertEqual((self.base / "extracted/dist/index.js").read_bytes(), b"throw Error('never execute');")

    def test_wrong_digest_and_missing_manifest_do_not_create_destination(self):
        file, digest = self.archive()
        with self.assertRaisesRegex(ValueError, "digest"):
            self.extract(file, "0" * 64)
        file, digest = self.archive(include_manifest=False)
        with self.assertRaisesRegex(ValueError, "package.json"):
            self.extract(file, digest)
        self.assertFalse((self.base / "extracted").exists())

    def test_rejects_escape_windows_ambiguity_and_foreign_root(self):
        for name in ["package/../outside", "/package/absolute", "package/a\\b", "package/C:/x", "other/file", "package/CON.txt", "package/trailing. ", "package/a//b"]:
            with self.subTest(name=name):
                file, digest = self.archive([(name, b"x", tarfile.REGTYPE)])
                with self.assertRaisesRegex(ValueError, "member|path|root"):
                    self.extract(file, digest)
                self.assertFalse((self.base / "extracted").exists())

    def test_rejects_duplicates_case_collisions_and_file_directory_conflicts(self):
        cases = [
            [("package/dist/index.js", b"other", tarfile.REGTYPE)],
            [("package/DIST/other.js", b"other", tarfile.REGTYPE)],
            [("package/dist", b"not a directory", tarfile.REGTYPE)],
        ]
        for rows in cases:
            with self.subTest(rows=rows):
                file, digest = self.archive(rows)
                with self.assertRaisesRegex(ValueError, "(?i)duplicate|collision|directory"):
                    self.extract(file, digest)
                self.assertFalse((self.base / "extracted").exists())

    def test_rejects_symlinks_hardlinks_and_special_members(self):
        for kind in [tarfile.SYMTYPE, tarfile.LNKTYPE, tarfile.FIFOTYPE]:
            file, digest = self.archive([("package/linked", b"", kind)])
            with self.assertRaisesRegex(ValueError, "type|link|special"):
                self.extract(file, digest)
        self.assertFalse((self.base / "extracted").exists())

    def test_existing_or_repository_destination_is_never_overwritten(self):
        file, digest = self.archive()
        with self.assertRaisesRegex(ValueError, "repository"):
            self.extract(file, digest, self.repo / "inside")
        destination = self.base / "existing"
        destination.mkdir()
        (destination / "keep").write_text("preserve", encoding="utf8")
        with self.assertRaises(FileExistsError):
            self.extract(file, digest, destination)
        self.assertEqual((destination / "keep").read_text(), "preserve")

    def test_archive_expansion_and_member_limits_precede_writes(self):
        file, digest = self.archive()
        for setting, limit in [("MAX_ARCHIVE", 1), ("MAX_EXPANDED", 1), ("MAX_MEMBERS", 1)]:
            with self.subTest(setting=setting), patch.object(module, setting, limit):
                with self.assertRaisesRegex(ValueError, "limit"):
                    self.extract(file, digest)
                self.assertFalse((self.base / "extracted").exists())

    def test_cli_reserves_receipt_and_records_failure_without_overwriting(self):
        file, digest = self.archive()
        script = Path(__file__).with_name("farm-docs-extract-package.py")
        receipt, destination = self.base / "receipt.json", self.base / "cli-output"
        args = [sys.executable, str(script), "--archive", str(file), "--sha256", digest,
                "--destination", str(destination), "--repository", str(self.repo), "--receipt", str(receipt)]
        success = subprocess.run(args, capture_output=True, text=True)
        self.assertEqual(success.returncode, 0, success.stderr)
        self.assertTrue(json.loads(receipt.read_text(encoding="utf8"))["success"])
        original_receipt = receipt.read_bytes()
        failed = subprocess.run(args, capture_output=True, text=True)
        self.assertNotEqual(failed.returncode, 0)
        self.assertEqual(receipt.read_bytes(), original_receipt)
        args[args.index(str(receipt))] = str(self.base / "failure.json")
        args[args.index(digest)] = "0" * 64
        args[args.index(str(destination))] = str(self.base / "failed-output")
        failed = subprocess.run(args, capture_output=True, text=True)
        self.assertNotEqual(failed.returncode, 0)
        self.assertFalse(json.loads((self.base / "failure.json").read_text(encoding="utf8"))["success"])
        self.assertFalse((self.base / "failed-output").exists())


if __name__ == "__main__":
    unittest.main()
