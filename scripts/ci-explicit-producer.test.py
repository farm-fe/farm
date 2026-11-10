"""Trusted needs bindings select producers without clock-based inference."""
# cspell:words importlib
import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('unit_outputs', Path(__file__).with_name('ci-unit-upload-outputs.test.py'))
f = importlib.util.module_from_spec(spec); spec.loader.exec_module(f)


class Contracts(unittest.TestCase):
    def fixture(self):
        data = f.f.fixture()
        for attempt, jobs in data[3].items():
            for job in jobs:
                job.update(head_sha=data[1]['head_sha'], run_attempt=attempt)
                job.pop('started_at', None); job.pop('completed_at', None)
                for step in job['steps']:
                    step.pop('started_at', None); step.pop('completed_at', None)
        return data

    def test_explicit_origin_ignores_all_clock_fields_and_changed_carried_ids(self):
        data = self.fixture(); api, run, _, jobs, *_ = data
        origin_job = next(j for j in jobs[1] if j['name'] == 'Test - macos-latest')
        current_job = next(j for j in jobs[2] if j['name'] == 'Test - macos-latest')
        current_job['id'] += 10000
        bound = f.outputs(data[4])['rust-tests:ci-test:darwin-arm64']
        producer, job = f.m.execution.explicit_producer(api, run, 'Test - macos-latest', bound)
        self.assertEqual(producer['run_attempt'], 1)
        self.assertEqual(job['id'], origin_job['id'])

    def test_unit_production_resolver_has_no_clock_dependency_with_trusted_outputs(self):
        data = self.fixture()
        values = f.outputs(data[4])
        for row in data[4].values(): row.pop('created_at', None)
        for job in data[3][2]: job['id'] += 10000
        plan, cells = f.m.unit_upload_outputs({'CI_UNIT_UPLOADS': __import__('json').dumps(values)})
        _, observations, proof = f.m.resolve(*data[:3], upload_outputs=cells, planner_output=plan)
        self.assertEqual(len(observations), 4)
        self.assertEqual(proof['plannerAttempt'], 1)

    def test_real_attempt_api_views_with_simulated_needs_bindings(self):
        import json
        observed = json.loads(Path(__file__).with_name('fixtures').joinpath('explicit-producer-api-observation.json').read_text())
        run = {'id': observed['runId'], 'run_attempt': 3, 'head_sha': observed['headSHA'],
               'path': '.github/workflows/ci.yaml', 'event': 'pull_request',
               'repository': {'id': 1}, 'head_repository': {'id': 1}}
        class API:
            def get(self, route):
                attempt = int(route.split('/attempts/')[1].split('/')[0])
                if '/jobs?' in route:
                    return {'jobs': observed['attempts'][str(attempt)]['jobs']}
                return {**run, 'run_attempt': attempt}
        for label, attempt, expected in [('macOS', 1, 126362153634), ('Windows', 1, 126374419345), ('Linux', 2, 126406306733)]:
            names = [row['name'] for row in observed['attempts'][str(attempt)]['jobs'] if row['id'] == expected]
            binding = {'artifact_id': '1', 'artifact_digest': 'a' * 64, 'run_attempt': str(attempt)}
            for view in observed['attempts'].values():
                self.assertEqual(view['total_count'], view['read_count'])
                for job in view['jobs']:
                    job.pop('started_at', None); job.pop('completed_at', None)
                    for step in job['steps']:
                        step.pop('started_at', None); step.pop('completed_at', None)
            _, job = f.m.execution.explicit_producer(API(), run, names[0], binding)
            self.assertEqual(job['id'], expected, label)

    def test_latest_failure_missing_ambiguous_or_wrong_source_reject(self):
        for defect in ['failure', 'cancelled', 'missing', 'duplicate', 'sha', 'run', 'attempt', 'old-failure']:
            data = self.fixture(); api, run, _, jobs, *_ = data
            row = next(j for j in jobs[2] if j['name'] == 'Test - macos-latest')
            bound = f.outputs(data[4])['rust-tests:ci-test:darwin-arm64']
            if defect in ['failure','cancelled']: row['conclusion'] = defect
            elif defect == 'missing': jobs[2].remove(row)
            elif defect == 'duplicate': jobs[2].append(copy.deepcopy(row))
            elif defect == 'sha': row['head_sha'] = 'e' * 40
            elif defect == 'run': row['run_id'] += 1
            elif defect == 'attempt': bound['run_attempt'] = '3'
            else: next(j for j in jobs[1] if j['name'] == row['name'])['conclusion'] = 'failure'
            with self.subTest(defect=defect), self.assertRaises(ValueError):
                f.m.execution.explicit_producer(api, run, 'Test - macos-latest', bound)

    def test_binding_inventory_and_formats_reject(self):
        for defect in ['none','unknown','id','digest','attempt']:
            data = self.fixture(); bound = f.outputs(data[4])['rust-tests:ci-test:darwin-arm64']
            if defect == 'none': bound = None
            elif defect == 'unknown': bound['job'] = 'self-asserted'
            else: bound[{'id':'artifact_id','digest':'artifact_digest','attempt':'run_attempt'}[defect]] = 0
            with self.subTest(defect=defect), self.assertRaises(ValueError):
                f.m.execution.explicit_producer(data[0], data[1], 'Test - macos-latest', bound)

if __name__ == '__main__': unittest.main()
