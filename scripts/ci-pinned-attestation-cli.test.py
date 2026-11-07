"""Pure extraction contracts; never provision or execute a fake hosted identity."""
# cspell:words importlib
import hashlib
import importlib.util
import io
from pathlib import Path
import tarfile
import unittest
spec=importlib.util.spec_from_file_location('pinned_cli',Path(__file__).with_name('ci-pinned-attestation-cli.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)


class Contracts(unittest.TestCase):
    def archive(self,kind=tarfile.REGTYPE,duplicate=False):
        output=io.BytesIO()
        with tarfile.open(fileobj=output,mode='w:gz') as archive:
            for _ in range(2 if duplicate else 1):
                item=tarfile.TarInfo('fixture/bin/gh');item.type=kind;item.size=5 if kind==tarfile.REGTYPE else 0
                archive.addfile(item,io.BytesIO(b'bytes') if item.size else None)
        return output.getvalue()

    def test_verified_regular_member_only(self):
        body=self.archive();digest=hashlib.sha256(body).hexdigest()
        self.assertEqual(m.unpack_verified_binary(body,digest,'fixture/bin/gh'),b'bytes')
        for data,sha,name in [(body,'0'*64,'fixture/bin/gh'),(body,digest,'missing')]:
            with self.assertRaises(ValueError):m.unpack_verified_binary(data,sha,name)
        for kind,duplicate in [(tarfile.SYMTYPE,False),(tarfile.REGTYPE,True)]:
            body=self.archive(kind,duplicate)
            with self.assertRaises(ValueError):m.unpack_verified_binary(body,hashlib.sha256(body).hexdigest(),'fixture/bin/gh')

    def test_production_entry_does_not_accept_external_verified_result(self):
        import os
        import tempfile
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            forged=Path(directory)/'verified.json';forged.write_text('{"verified":true}')
            with patch.dict(os.environ,{'FARM_VERIFIED_RESULT':str(forged),'GH_ATTESTATION_VERIFIED_RESULT':str(forged)},clear=True):
                with patch.object(m.urllib.request,'urlopen',side_effect=AssertionError('Must reject before network')):
                    with self.assertRaisesRegex(PermissionError,'hosted Linux x64'):
                        m.verify_on_host(forged,forged,{'verified':True})

    def test_production_release_pin_is_exact(self):
        self.assertEqual(m.URL,'https://github.com/cli/cli/releases/download/v2.103.3/gh_2.103.3_linux_amd64.tar.gz')
        self.assertEqual(m.SIZE,15031972)
        self.assertEqual(m.SHA256,'4c806978ffce1cec70853302a56d70ac6da4f19bae7adca4e1a8ab00633dd91d')


if __name__=='__main__':unittest.main()
