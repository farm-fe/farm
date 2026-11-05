"""Optional local verification of historical performance archives; not a release gate."""
import ast
import re
import math
import urllib.parse
import hashlib
import io
import json
import stat
import zipfile
import importlib.util
from pathlib import Path
spec = importlib.util.spec_from_file_location('local_functional_helpers', Path(__file__).with_name('incremental-release-functional.py'))
functional = importlib.util.module_from_spec(spec); spec.loader.exec_module(functional)
entry, check, exact_plan = functional.entry, functional.check, functional.exact_plan

def fetch_member(transport, run, name, member):
    response = transport.get('/actions/runs/' + str(run['id']) + '/artifacts?per_page=2&name=' + urllib.parse.quote(name, safe=''))
    rows = response['artifacts']
    check(response.get('total_count') == 1 and len(rows) == 1, 'Exact performance artifact missing')
    artifact = transport.get('/actions/artifacts/' + str(rows[0]['id']))
    check(artifact['name'] == name and artifact['workflow_run']['id'] == run['id'] and artifact['expired'] is False,
          'Performance artifact producer differs')
    return exact_plan(transport.archive(artifact['id']), artifact, member), artifact


def verify_measurement_source(transport, run, source, summary, row, proof, observations=None):
    check(isinstance(proof, dict) and proof.get('profile') == 'ci' and proof.get('workflowPath') == '.github/workflows/ci.yaml', 'Measurement chain differs')
    systems = {'linux': ('linux-x64-gnu', 'ubuntu-24.04', 'Primary Linux'), 'darwin': ('darwin-arm64', 'macos-latest', 'Primary macOS'), 'win32': ('win32-x64-msvc', 'windows-latest', 'Primary Windows')}
    abi, runner, platform = systems[row['metadata']['os']]
    check(proof.get('abi') == abi and proof.get('runId') == run['id'] and proof.get('producerSHA') == source['sha'] and proof.get('testedTree') == source['tree'], 'Measurement source differs')
    for key in ['runAttempt', 'nativeAttempt']:
        check(type(proof.get(key)) is int and 0 < proof[key] <= run['run_attempt'], 'Invalid measurement attempt')
    check(type(run.get('run_attempt')) is int and 0 < run['run_attempt'] <= 32, 'Measurement attempts exceed bound')
    observations = {} if observations is None else observations
    def observed(attempt):
        if attempt in observations: return observations[attempt]
        original = transport.get('/actions/runs/' + str(run['id']) + '/attempts/' + str(attempt))
        check(original['id'] == run['id'] and original['run_attempt'] == attempt and original['head_sha'] == run['head_sha'] and original['path'] == run['path'] and original['event'] == run['event'] and original['repository']['id'] == run['repository']['id'], 'Original measurement run differs')
        rows = []
        for page in range(1, 21):
            batch = transport.get('/actions/runs/' + str(run['id']) + '/attempts/' + str(attempt) + '/jobs?per_page=100&page=' + str(page))['jobs']
            rows.extend(batch)
            if len(batch) < 100: break
        else: raise ValueError('Measurement jobs exceed bound')
        observations[attempt] = (original, rows)
        return original, rows
    def actual(attempt, name, job):
        check(type(job.get('id')) is int and job['id'] > 0 and job.get('run_id') == run['id'], 'Measurement job identity differs')
        check(job.get('started_at') and job.get('completed_at') and job.get('status') == 'completed' and job.get('conclusion') == 'success', 'Latest actual measurement producer did not succeed')
        if attempt == 1: return True
        _, previous = observed(attempt - 1)
        matching = [item for item in previous if item['name'] == name]
        check(len(matching) <= 1, 'Ambiguous previous measurement producer')
        if not matching: return True
        prior = matching[0]
        if (prior.get('started_at'), prior.get('completed_at')) == (job.get('started_at'), job.get('completed_at')):
            check(entry.execution.same_carried_execution(job, prior, run, attempt), 'Unproven carried measurement producer')
            return False
        check(not prior.get('completed_at') or job['started_at'] >= prior['completed_at'], 'Producer execution times overlap across attempts')
        return True
    def producer(attempt, name, job_id):
        _, rows = observed(attempt)
        job = entry.evidence.successful_job(rows, name)
        check(job['id'] == job_id and actual(attempt, name, job), 'Carried job is not a new producer')
        for later in range(attempt + 1, run['run_attempt'] + 1):
            _, later_jobs = observed(later)
            matching = [item for item in later_jobs if item['name'] == name]
            check(len(matching) <= 1, 'Ambiguous later measurement producer')
            check(not any(actual(later, name, item) for item in matching), 'A newer actual producer cannot be hidden')
        return job
    def member(artifact_id, digest, name, job, filename):
        artifact = transport.get('/actions/artifacts/' + str(artifact_id))
        check(artifact['id'] == artifact_id and artifact['digest'] == digest and artifact['name'] == name and not artifact['expired'] and artifact['workflow_run']['id'] == run['id'] and artifact['workflow_run']['repository_id'] == run['repository']['id'] and artifact['workflow_run']['head_sha'] == run['head_sha'], 'Measurement artifact differs')
        check(job['started_at'] <= artifact['created_at'] <= job['completed_at'], 'Measurement upload outside producer')
        body = transport.archive(artifact_id)
        check(0 < len(body) <= 16*1024*1024 and len(body) == artifact['size_in_bytes'] and digest == 'sha256:' + hashlib.sha256(body).hexdigest(), 'Measurement archive bytes differ')
        with zipfile.ZipFile(io.BytesIO(body)) as archive:
            entries = archive.infolist(); names = [item.filename for item in entries]
            check(len(entries) <= 500 and len(set(names)) == len(names), 'Ambiguous measurement archive')
            check(all(not item.flag_bits & 1 and stat.S_IFMT(item.external_attr >> 16) in [0, stat.S_IFREG, stat.S_IFDIR] and not item.filename.startswith('/') and '\\' not in item.filename and ':' not in item.filename and all(part not in ['.', '..', ''] for part in item.filename.rstrip('/').split('/')) for item in entries), 'Unsafe measurement archive')
            item = archive.getinfo(filename); check(0 < item.file_size <= 2*1024*1024, 'Measurement member exceeds bound')
            content = archive.read(item)
            return json.loads(content.decode('utf-8')), hashlib.sha256(content).hexdigest()
    job = producer(proof['runAttempt'], 'Commerce Performance (' + runner + ')', proof['jobId'])
    raw, digest = member(proof['artifactId'], proof['artifactDigest'], 'performance-' + source['sha'] + '-' + abi + '-attempt-' + str(proof['runAttempt']), job, 'summary.json')
    check(digest == proof['rawSummarySha256'] and raw.get('status') == 'complete' and raw.get('protocol') == summary['protocol'], 'Raw measurement differs')
    for key, value in row['metadata'].items(): check(raw.get('metadata', {}).get(key) == value, 'Compacted measurement metadata differs')
    check(raw['metadata']['artifactProducerSHA'] == source['sha'] and raw['metadata']['harnessSha256'] == proof['harnessSha256'], 'Measurement input differs')
    check(raw.get('cells') == row['cells'] and raw.get('comparison', {}).get('mode') == row['comparison']['mode'] and raw.get('comparison', {}).get('baseCells', []) == row['comparison']['baseCells'], 'Compacted measurement cells differ')
    native_kind = proof.get('nativeKind', 'build')
    check(native_kind in ['build','materialization'], 'Unknown native source kind')
    reused = native_kind == 'materialization'
    check(not reused or abi == 'linux-x64-gnu', 'Unsupported native reuse platform')
    native = producer(proof['nativeAttempt'], 'Type Check' if reused else platform + ' / native / Build and Upload User Artifacts - ' + abi, proof['nativeJobId'])
    name = 'ci-consumer-native-proof-' + str(run['id']) + '-' + str(proof['nativeAttempt']) if reused else 'native-shadow-' + source['sha'] + '-' + abi + '-' + str(proof['nativeAttempt'])
    products, _ = member(proof['nativeArtifactId'], proof['nativeArtifactDigest'], name, native, 'consumer.json' if reused else 'products.json')
    check(products.get('profile') == 'ci' and products.get('abi') == abi and str(products.get('runId')) == str(run['id']), 'Measured native producer differs')
    if reused:
        claim = raw.get('metadata',{}).get('nativeReuse')
        check(isinstance(claim,dict) and claim.get('chain') == 'ci' and claim.get('consumerSHA') == source['sha'] and claim.get('acceptanceInherited') is False, 'Measured reuse claim differs')
        check(products.get('kind') == 'ci-consumer-materialization-not-a-new-build' and products.get('chain') == 'ci'
              and products.get('consumerSHA') == source['sha'] and products.get('consumerTree') == source['tree']
              and str(products.get('attempt')) == str(proof['nativeAttempt']) and products.get('acceptanceInherited') is False
              and products.get('originContext',{}).get('profile') == 'ci'
              and products.get('originBuildSHA') == claim.get('originBuildSHA')
              and str(products.get('originArtifactId')) == str(claim.get('originArtifactId')), 'Measured original materialization differs')
    else:
        check(raw.get('metadata',{}).get('nativeReuse') is None and products.get('originBuildSHA') == source['sha'] and str(products.get('runAttempt')) == str(proof['nativeAttempt']) and products.get('repository') == run['repository']['full_name'], 'Measured native build producer differs')
    for key in ['jobId', 'artifactId', 'nativeJobId', 'nativeArtifactId']:
        check(type(proof.get(key)) is int and proof[key] > 0, 'Invalid measurement producer ID')
    records = products.get('records', []); owners = {item['owner']: item for item in records}
    check(len(owners) == len(records) and raw.get('native', {}).get('artifacts'), 'Measured native inventory absent')
    policy_bytes = source['files']['scripts/performance-cohorts.mjs']
    check(isinstance(policy_bytes, bytes), 'Measured cohort policy must be source bytes')
    policy_text = policy_bytes.decode('utf-8', errors='strict')
    declaration = re.search(r'export const nativePlugins = (\[[^;]+\]);', policy_text)
    check(declaration is not None, 'Measured cohort policy missing')
    plugins = ast.literal_eval(declaration[1])
    check(isinstance(plugins, list) and plugins and len(set(plugins)) == len(plugins) and all(isinstance(item, str) and re.fullmatch(r'[a-z-]+', item) for item in plugins), 'Measured cohort policy differs')
    required = {'@farmfe/core', *['@farmfe/plugin-' + item for item in plugins]}
    check(set(raw['native']['artifacts']) == required, 'Incomplete measured native cohort')
    for owner, actual in raw['native']['artifacts'].items():
        check(owner in owners and owners[owner]['sha256'] == actual['sha256'] and owners[owner]['bytes'] == actual['bytes'], 'Measured native bytes differ')


def verify_performance(transport, run, jobs, source, plan):
    aggregation, producer = entry.execution.actual_producer(transport, run, 'Aggregate informational performance report', jobs)
    summary, artifact = fetch_member(transport, run, 'performance-pr-summary-' + str(aggregation['run_attempt']), 'summary.json')
    check(producer['started_at'] <= artifact['created_at'] <= producer['completed_at'], 'Performance artifact outside successful attempt')
    check(summary.get('schemaVersion') in [2, 3] and summary.get('protocol') == 'commerce-business-2048-farm-supervised-v2', 'Performance protocol differs')
    check(summary.get('runId') == run['id'] and summary.get('runAttempt') == aggregation['run_attempt']
          and summary.get('repositoryId') == run['repository']['id'] and summary.get('event') == run['event']
          and summary.get('headSHA') == run['head_sha'] and summary.get('baseSHA') == plan.get('baseSHA'), 'Performance source identity differs')
    check(summary.get('prNumber') is None if run['event'] == 'push' else type(summary.get('prNumber')) is int, 'Performance event association differs')
    reports = summary.get('reports', [])
    check(len(reports) == 3 and {(row.get('metadata', {}).get('os'), row.get('metadata', {}).get('arch')) for row in reports}
          == {('linux', 'x64'), ('darwin', 'arm64'), ('win32', 'x64')}, 'Three performance platforms required')
    if summary['schemaVersion'] == 3:
        proofs = summary.get('measurementSources')
        check(isinstance(proofs, list) and len(proofs) == len(reports), 'Measurement sources missing')
        observations = {}
        for row, proof in zip(reports, proofs): verify_measurement_source(transport, run, source, summary, row, proof, observations)
    else:
        check(run['run_attempt'] == 1, 'Legacy summary cannot authorize mixed attempts')
    for row in reports:
        meta = row['metadata']
        check(row.get('status') == 'complete' and meta.get('revision') == source['sha'] and meta.get('baseSHA') == summary['baseSHA'], 'Incomplete or stale performance report')
        for key in ['fixtureSha256', 'lockSha256']:
            entry.chain.exact(meta.get(key), 64)
        cells = row.get('cells', [])
        check(len(cells) == 2 and {cell.get('cacheMode') for cell in cells} == {'cold', 'warm'}, 'Cold and warm performance samples required')
        for cell in cells:
            check(type(cell.get('samples')) is int and cell['samples'] >= 3 and type(cell.get('validSamples')) is int
                  and cell['validSamples'] == cell['samples'], 'Incomplete actual performance samples')
            check(all(type(cell.get(key)) in [int, float] and math.isfinite(cell[key]) and cell[key] >= 0 for key in ['medianMs', 'iqrMs'])
                  and cell['medianMs'] > 0, 'Invalid actual performance values')
    return summary
