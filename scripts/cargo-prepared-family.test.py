"""Prepared Cargo family planning; no registry writes or network."""
import copy
import importlib.util
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('cargo_source_release',Path(__file__).with_name('cargo-source-release.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class PreparedFamily(unittest.TestCase):
    def fixture(self):
        rows=[{'name':name,'version':'2.0.0','manifestPath':name+'/Cargo.toml'} for name in ['retained','missing']]
        files={row['manifestPath']:('[package]\nname="'+row['name']+'"\nversion="2.0.0"\n').encode() for row in rows}
        return files, {'cargo':rows}, [{'name':'retained','version':'2.0.0','sha256':'a'*64}]
    def test_same_versions_on_base_still_select_unpublished_family(self):
        files,plan,retained=self.fixture()
        self.assertEqual(m.select_prepared_packages(files,plan,retained,lambda row:'a'*64),[plan['cargo'][1]])
    def test_no_baseline_or_retained_versions_selects_full_family(self):
        files,plan,_=self.fixture()
        self.assertEqual(m.select_prepared_packages(files,plan,[],lambda row:self.fail('unexpected registry call')),plan['cargo'])
    def test_missing_or_conflicting_retained_version_fails_before_upload(self):
        files,plan,retained=self.fixture()
        for checksum in [None,'b'*64]:
            with self.assertRaisesRegex(ValueError,'Retained Cargo archive'):m.select_prepared_packages(files,plan,retained,lambda row:checksum)
    def test_source_version_private_duplicate_and_foreign_retention_rejected(self):
        files,plan,retained=self.fixture()
        wrong=dict(files);wrong['missing/Cargo.toml']=wrong['missing/Cargo.toml'].replace(b'2.0.0',b'2.0.1')
        with self.assertRaisesRegex(ValueError,'source version'):m.select_prepared_packages(wrong,plan,retained,lambda row:'a'*64)
        wrong=dict(files);wrong['missing/Cargo.toml']+=b'publish=false\n'
        with self.assertRaisesRegex(ValueError,'Private'):m.select_prepared_packages(wrong,plan,retained,lambda row:'a'*64)
        duplicate=copy.deepcopy(plan);duplicate['cargo'].append(duplicate['cargo'][0])
        with self.assertRaisesRegex(ValueError,'Duplicate'):m.select_prepared_packages(files,duplicate,retained,lambda row:'a'*64)
        with self.assertRaisesRegex(ValueError,'outside'):m.select_prepared_packages(files,plan,[{**retained[0],'name':'foreign'}],lambda row:'a'*64)
if __name__=='__main__':unittest.main(verbosity=2)
