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

    def test_other_files_fields_locations_errors_and_duplicates_still_fail(self):
        source,rows=self.fixture()
        for field,value in [('filepath','other.yaml'),('line',999),('column',0),('kind','syntax-check'),('message','unrecognized input')]:
            row={**rows[0],field:value}
            with self.subTest(field=field):self.assertEqual(m.classify([row],source)['errors'],[row])
        self.assertEqual(m.classify(rows+[rows[0]],source)['errors'],[rows[0]])
        self.assertTrue(m.classify(rows,source.replace('job.workflow_sha','job.workflow_sha_typo'))['errors'])
        self.assertTrue(m.classify([None],source)['errors'])


if __name__=='__main__':unittest.main()
