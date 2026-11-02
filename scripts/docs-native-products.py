"""Same-run Docs delivery, never an independent cross-run reuse authority."""
import argparse
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import stat
import tomllib
import urllib.error
import zipfile

spec = importlib.util.spec_from_file_location('docs_formal', Path(__file__).with_name('formal-native-products.py'))
formal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
execution, check = formal.execution, formal.check
PLUGINS = 'replace-dirname react mdx ssr-css syntax-highlight sass tailwindcss docs docs-snapshot docs-seo docs-snippets'.split()
OWNERS = ['@farmfe/core'] + ['@farmfe/plugin-' + name for name in PLUGINS]
PLATFORMS = {'linux-x64-gnu': ('linux', 'x64'), 'win32-x64-msvc': ('win32', 'x64'), 'darwin-arm64': ('darwin', 'arm64')}
UPLOAD = 'Upload same-run Docs native products'
MAX = 512 * 1024 * 1024


def inputs(source, abi):
    check(abi in PLATFORMS, 'Unsupported Docs target')
    products = formal.product_inputs(source, abi)
    check(all(owner in products for owner in OWNERS), 'Incomplete Docs native closure')
    return {owner: products[owner] for owner in OWNERS}


def capture_identity(capture, source, abi, run_id, attempt):
    check(capture.get('originBuildSHA') == source['sha'] and str(capture.get('runId')) == str(run_id)
          and str(capture.get('runAttempt')) == str(attempt), 'Docs producer source or attempt differs')
    check(capture.get('abi') == abi and capture.get('profile') == 'ci', 'Docs producer role or target differs')
    runtime = capture.get('runtimeCapture') or {}
    check(runtime.get('abi') == abi and runtime.get('target') == formal.paths.TARGETS[abi]
          and runtime.get('profile') == 'ci', 'Docs measured target or role differs')
    check(runtime.get('rustc') and runtime.get('cargo') and runtime.get('runtime', {}).get('node') == 'v22.20.0', 'Docs measured tools missing')
    check(re.fullmatch('[a-f0-9]{64}', runtime.get('formalFlagsHash') or ''), 'Docs formal flags missing')
    channel = tomllib.loads(source['files']['rust-toolchain.toml'].decode())['toolchain']['channel']
    check(runtime.get('activeToolchain', '').startswith(channel + '-') and runtime.get('errors') == []
          and runtime.get('unboundBuildVariables') == [], 'Docs compiler policy or measured flags incomplete')
    # This is delivery of the exact current producer, not compatibility with a
    # different compiler host. The consumer does not compile reused products.
    # Cross-run decisions remain exclusively inside the formal producer resolver.


def produce(source, abi, capture, root, run_id, attempt):
    capture_identity(capture, source, abi, run_id, attempt)
    wanted = inputs(source, abi)
    captured = {row['owner']: row for row in capture['records']}
    check(len(captured) == len(capture['records']), 'Duplicate captured owner')
    records, bodies = [], {}
    for owner, expected in wanted.items():
        row = captured[owner]
        check(all(row[key] == expected[key] for key in ['file', 'name']), 'Docs capture destination differs')
        body = formal.workspace_path(root, row['file']).read_bytes()
        check(len(body) == row['bytes'] and hashlib.sha256(body).hexdigest() == row['sha256'], 'Docs captured bytes changed')
        check(formal.native_matches(body, abi), 'Docs native architecture differs')
        member = hashlib.sha256(owner.encode()).hexdigest() + '.bin'
        bodies[member] = body
        records.append({**expected, 'member': member, 'bytes': len(body), 'sha256': row['sha256']})
    check(sum(map(len, bodies.values())) < MAX - 1024 * 1024, 'Docs bundle too large')
    return {'schemaVersion': 1, 'kind': 'same-run-docs-native', 'sourceSHA': source['sha'], 'sourceTree': source['tree'],
            'runId': run_id, 'attempt': attempt, 'abi': abi, 'capture': capture, 'records': records}, bodies


def inspect(body, artifact, run, job, source, abi):
    execution.upload_binding(job, UPLOAD, artifact)
    check(artifact['workflow_run']['id'] == run['id'] and artifact['expired'] is False, 'Docs artifact origin differs')
    check(artifact['name'] == f"docs-native-{source['sha']}-{abi}-{run['run_attempt']}", 'Docs artifact identity differs')
    check(0 < len(body) <= MAX and len(body) == artifact['size_in_bytes']
          and artifact['digest'] == 'sha256:' + hashlib.sha256(body).hexdigest(), 'Docs archive digest differs')
    wanted = inputs(source, abi)
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        entries = archive.infolist()
        check(len(entries) == len(OWNERS) + 1 and sum(row.file_size for row in entries) <= MAX, 'Docs archive bounds differ')
        formal.paths.validate_member_set([row.filename for row in entries], abi)
        check(all(not row.is_dir() and stat.S_IFMT(row.external_attr >> 16) in [0, stat.S_IFREG] for row in entries), 'Docs special member rejected')
        check(archive.getinfo('manifest.json').file_size < 1024 * 1024, 'Docs manifest too large')
        manifest = json.loads(archive.read('manifest.json'))
        check(all(manifest.get(key) == value for key, value in {'schemaVersion': 1, 'kind': 'same-run-docs-native',
              'sourceSHA': source['sha'], 'sourceTree': source['tree'], 'runId': run['id'],
              'attempt': run['run_attempt'], 'abi': abi}.items()), 'Docs manifest provenance differs')
        capture_identity(manifest['capture'], source, abi, run['id'], run['run_attempt'])
        records = manifest['records']
        check(len(records) == len(wanted) and {row['owner'] for row in records} == set(wanted), 'Docs owner inventory differs')
        check({row.filename for row in entries} == {'manifest.json'} | {row['member'] for row in records}, 'Unexpected Docs archive members')
        reused, missing = {}, []
        for row in records:
            expected = wanted[row['owner']]
            data = archive.read(formal.paths.safe_member(row['member']))
            check(len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256'], 'Docs member digest differs')
            check(formal.native_matches(data, abi), 'Docs binary target differs')
            check(all(row[key] == expected[key] for key in ['owner', 'name', 'file', 'abi', 'version']), 'Docs owner destination differs')
            if any(row.get(key) != expected[key] for key in ['recipe', 'inputHash', 'abiCohort', 'contentKey']):
                missing.append(row['owner'])
            else:
                reused[row['owner']] = (expected['file'], data)
    return reused, missing


def resolve(transport, run, source, abi):
    check(run['path'] == '.github/workflows/ci.yaml' and run['event'] in ['push', 'pull_request', 'workflow_dispatch'], 'Unexpected Docs producer workflow')
    check(source['sha'] == run['head_sha'] or run['event'] == 'pull_request' and run['head_sha'] in source['parents'], 'Docs source differs from run')
    if run['repository']['id'] != run['head_repository']['id']:
        return {}, list(OWNERS), 'fork-source-build'
    prefix = {'linux-x64-gnu': 'Primary Linux', 'win32-x64-msvc': 'Primary Windows'}[abi]
    producer, job = execution.actual_producer(transport, run, prefix + ' / native / Build and Upload User Artifacts - ' + abi)
    name = f"docs-native-{source['sha']}-{abi}-{producer['run_attempt']}"
    response = transport.get('/actions/runs/' + str(run['id']) + '/artifacts?per_page=2&name=' + name)
    rows = [row for row in response['artifacts'] if row['name'] == name]
    check(response.get('total_count', len(rows)) <= 1 and len(rows) <= 1, 'Ambiguous Docs artifact')
    if not rows or rows[0]['expired']: return {}, list(OWNERS), 'missing-or-expired'
    try:
        artifact = transport.get('/actions/artifacts/' + str(rows[0]['id']))
        if artifact['expired']: return {}, list(OWNERS), 'expired'
        body = transport.archive(artifact['id'])
    except urllib.error.HTTPError as error:
        if error.code in [404, 410]: return {}, list(OWNERS), 'artifact-no-longer-available'
        raise
    except ValueError as error:
        # The shared Actions transport normalizes GitHub API HTTP failures.
        # Match only explicit disappearance, never malformed proof or denial.
        if str(error) in [f'GitHub {kind} unavailable (HTTP {code})'
                          for kind in ['evidence', 'artifact read'] for code in [404, 410]]:
            return {}, list(OWNERS), 'artifact-no-longer-available'
        raise
    reused, missing = inspect(body, artifact, producer, job, source, abi)
    return reused, missing, 'verified-current-producer'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['produce', 'consume'])
    parser.add_argument('--abi', required=True, choices=PLATFORMS)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    env = os.environ
    source = formal.evidence.source_reader(env['GITHUB_SHA'])
    if args.mode == 'produce':
        check(env.get('GITHUB_ACTIONS') == 'true' and env.get('GITHUB_JOB') == 'build', 'Unexpected Docs sealing job')
        capture = json.loads(Path(f'.ci-native-host-proof/{args.abi}/products.json').read_text())
        manifest, bodies = produce(source, args.abi, capture, Path.cwd(), int(env['GITHUB_RUN_ID']), int(env['GITHUB_RUN_ATTEMPT']))
        output = Path(args.output); output.mkdir(exist_ok=False)
        (output / 'manifest.json').write_text(json.dumps(manifest, sort_keys=True), encoding='utf-8')
        for member, body in bodies.items(): (output / member).write_bytes(body)
        return
    check(env.get('GITHUB_JOB') == 'docs' and env.get('GITHUB_ACTIONS') == 'true', 'Unexpected Docs consumer job')
    reused, missing, reason = {}, list(OWNERS), 'standalone-source-build'
    if env.get('FARM_DOCS_CONSUME_NATIVE') == 'true':
        transport = formal.evidence.transport_module.GitHubReadOnly(env['GITHUB_REPOSITORY'], env['GH_TOKEN'], allow_archive=True)
        run = transport.get('/actions/runs/' + env['GITHUB_RUN_ID'] + '/attempts/' + env['GITHUB_RUN_ATTEMPT'])
        check(run['id'] == int(env['GITHUB_RUN_ID']) and run['run_attempt'] == int(env['GITHUB_RUN_ATTEMPT'])
              and run['repository']['id'] == int(env['GITHUB_REPOSITORY_ID']), 'Docs executing run differs')
        reused, missing, reason = resolve(transport, run, source, args.abi)
    for owner, (file, body) in reused.items():
        target = formal.workspace_path(Path.cwd(), file, create_parents=True)
        target.write_bytes(body)
    plan = {'schemaVersion': 1, 'abi': args.abi, 'sourceSHA': source['sha'], 'sourceTree': source['tree'],
            'missing': missing, 'reused': {owner: {'file': file, 'sha256': hashlib.sha256(body).hexdigest()} for owner, (file, body) in reused.items()}, 'reason': reason}
    body = json.dumps(plan, sort_keys=True).encode()
    Path(args.output).write_bytes(body)
    with Path(env['GITHUB_OUTPUT']).open('a', encoding='utf-8') as stream:
        stream.write('plan_hash=' + hashlib.sha256(body).hexdigest() + '\n')
    print(json.dumps({'reused': list(reused), 'build': missing, 'reason': reason}))


if __name__ == '__main__': main()
