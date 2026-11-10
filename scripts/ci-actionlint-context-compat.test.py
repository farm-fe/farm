"""Only the exact documented job-context diagnostics are compatible."""
# cspell:words importlib
import importlib.util
import unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('compat',Path(__file__).with_name('ci-actionlint-context-compat.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)


class Contracts(unittest.TestCase):
    def fixture(self):
        lines=['  '+line for line in m.ALLOWED]
        rows=[{'filepath':m.WORKFLOW,'line':index+1,'column':line.index('job.')+1,'kind':'expression',
               'message':'property "'+m.ALLOWED[line.strip()]+'" is not defined in object type '+m.OBJECT} for index,line in enumerate(lines)]
        return '\n'.join(lines),rows

    def test_exact_three_documented_contexts_are_reported_not_erased(self):
        source,rows=self.fixture();value=m.classify(rows,source)
        self.assertEqual(value['documentedContextCompatibility'],rows)
        self.assertEqual(value['errors'],[])
        self.assertEqual(m.classify([],source),{'documentedContextCompatibility':[],'errors':[]})

    def test_release_signer_uses_the_same_exact_documented_contexts_without_hiding_other_errors(self):
        source,rows=self.fixture()
        release='.github/workflows/release-provenance-signer.yaml'
        renamed=[{**row,'filepath':release} for row in rows]
        report=m.classify(renamed,{release:source})
        self.assertEqual(len(report['documentedContextCompatibility']),3)
        self.assertEqual(report['errors'],[])
        self.assertEqual(m.classify(renamed,{release:source.replace('job.workflow_sha','github.workflow_sha')})['errors'],renamed[:2])

    def test_queue_compatibility_requires_exact_non_cancelling_release_policy(self):
        name='.github/workflows/incremental-release.yaml'
        source='concurrency:\n  group: release-main\n  cancel-in-progress: false\n  queue: max\n'
        row={'filepath':name,'line':4,'column':3,'kind':'syntax-check','message':'unexpected key "queue" for "concurrency" section. expected one of "cancel-in-progress", "group"'}
        self.assertEqual(m.classify([row],{name:source})['errors'],[])
        for altered in [source.replace('false','true'),source.replace('max','single'),source.replace('concurrency:','other:')]:
            self.assertEqual(m.classify([row],{name:altered})['errors'],[row])
        self.assertEqual(m.classify([row,row],{name:source})['errors'],[row])
        unrelated={**row,'line':6}
        self.assertEqual(m.classify([unrelated],{name:source+'other:\n  queue: max\n'})['errors'],[unrelated])
        self.assertEqual(m.classify([{**row,'filepath':'other.yaml'}],{'other.yaml':source})['errors'],[{**row,'filepath':'other.yaml'}])

    def test_other_files_fields_locations_errors_and_duplicates_still_fail(self):
        source,rows=self.fixture()
        for field,value in [('filepath','other.yaml'),('line',999),('column',0),('kind','syntax-check'),('message','unrecognized input')]:
            row={**rows[0],field:value}
            with self.subTest(field=field):self.assertEqual(m.classify([row],source)['errors'],[row])
        self.assertEqual(m.classify(rows+[rows[0]],source)['errors'],[rows[0]])
        self.assertTrue(m.classify(rows,source.replace('job.workflow_sha','job.workflow_sha_typo'))['errors'])
        self.assertTrue(m.classify([None],source)['errors'])


if __name__=='__main__':unittest.main()
