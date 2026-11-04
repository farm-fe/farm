"""Per-product, read-only main formal artifact consumption; no test inheritance."""
import argparse
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tomllib
import zipfile


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module


evidence = load('formal_evidence', 'incremental-chain-evidence.py')
execution = load('formal_execution', 'ci-execution-evidence.py')
paths = load('formal_paths', 'ci-native-proof.py')
chain = evidence.chain_plan
check = evidence.check
WORKFLOW = '.github/workflows/incremental-release.yaml'
UPLOAD = 'Upload formal native product catalog'
MAX = 512 * 1024 ** 2


def native_matches(body, abi):
    os_name, arch, *_ = abi.split('-')
    if os_name == 'darwin':
        return len(body) >= 8 and int.from_bytes(body[:4], 'little') == 0xfeedfacf and int.from_bytes(body[4:8], 'little') == {'x64': 0x1000007, 'arm64': 0x100000c}.get(arch)
    if os_name == 'linux':
        return len(body) >= 20 and body[:6] == b'\x7fELF\x02\x01' and int.from_bytes(body[18:20], 'little') == {'x64': 62, 'arm64': 183}.get(arch)
    if os_name == 'win32' and len(body) >= 64 and body[:2] == b'MZ':
        offset = int.from_bytes(body[60:64], 'little')
        return offset + 6 <= len(body) and body[offset:offset+4] == b'PE\0\0' and int.from_bytes(body[offset+4:offset+6], 'little') == {'x64': 0x8664, 'ia32': 0x14c, 'arm64': 0xaa64}.get(arch)
    return False


def workspace_path(root, relative, create_parents=False):
    paths.safe_member(relative)
    root = Path(root).resolve(strict=True)
    current = root
    parts = relative.split('/')
    for index, part in enumerate(parts):
        current = current / part
        check(not current.is_symlink() and not current.is_junction(), 'Native path redirects')
        if index < len(parts) - 1:
            if not current.exists() and create_parents: current.mkdir()
            check(current.is_dir(), 'Native parent missing or not a directory')
        check(current.resolve().is_relative_to(root), 'Native path escaped workspace')
    return current


def consumer_identity(source, abi, env):
    check(env.get('GITHUB_ACTIONS') == 'true', 'Actions consumer identity required')
    check(env.get('GITHUB_SHA') == source['sha'], 'Consumer checkout changed')
    check(env.get('GITHUB_JOB'), 'Consumer job identity missing')
    value = {key: env.get(key) for key in ['GITHUB_REPOSITORY', 'GITHUB_REPOSITORY_ID', 'GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_JOB', 'GITHUB_WORKFLOW_REF']}
    for key in ['GITHUB_REPOSITORY_ID', 'GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT']:
        check(str(value[key]).isdigit() and int(value[key]) > 0, 'Consumer numeric identity missing')
    return {**value, 'sourceSHA': source['sha'], 'sourceTree': source['tree'], 'abi': abi}


def product_inputs(source, abi):
    check(abi in paths.TARGETS, 'Unknown formal target')
    nodes, issues = chain.repository_graph(source['files'])
    check(not issues, 'Unresolved formal product dependencies')
    family = chain.independent_family(source['files'], nodes)
    snapshot = chain.snapshot_records(source['files'], source['modes'])
    shared = [key for key, node in nodes.items() if node['kind'] == 'cargo' and node['name'] in
              ['farmfe_core', 'farmfe_toolkit', 'farmfe_toolkit_plugin_types', 'farmfe_macro_plugin']]

    def closure(roots, owner_directory=None):
        seen, pending = set(), list(roots)
        while pending:
            key = pending.pop()
            if key in seen: continue
            check(key in nodes, 'Missing native dependency node')
            seen.add(key); pending.extend(nodes[key]['preparationDeps'])
        records = {}
        for file, value in snapshot.items():
            for key in seen:
                node = nodes[key]
                if node['kind'] == 'input':
                    match = any(file.startswith(rule) if rule.endswith('/') else file == rule for rule in node['paths'])
                else:
                    match = file == node['manifest'] or file.startswith(node['directory'] + '/')
                    # Nested platform package outputs are separately version-bound.
                    if '/npm/' in file and '/npm/' not in node['directory']: match = False
                if match: records[file] = value; break
        if owner_directory:
            for file, value in snapshot.items():
                if file.startswith(owner_directory + '/') and '/npm/' not in file:
                    records[file] = value
        return chain.digest(records)

    cohort = closure(shared)
    result = {}
    for row in family.values():
        if row.get('platform') != abi: continue
        owner = family['npm:' + row['owner']]
        directory = str(Path(owner['manifestPath']).parent).replace('\\', '/')
        file = ('packages/core/binding/farm.' + abi + '.node' if row['owner'] == '@farmfe/core' else
                'packages/create-farm/create-farm.' + abi + '.node' if row['owner'] == 'create-farm' else directory + '/npm/' + abi + '/index.farm')
        core = row['owner'] == '@farmfe/core'
        # Fingerprint the native producer, not JS preparation cycles. The npm
        # wrapper's own inputs/version remain bound, and plugin-tools is part of
        # sourceRecipe. JS dependencies still govern JS build/test selection.
        producer = 'crates/node' if core else directory
        check('cargo:' + producer in nodes, 'Native Cargo producer missing')
        profile = 'release-publish' if core and abi not in ['win32-ia32-msvc', 'win32-arm64-msvc'] else 'release'
        recipe = {'schemaVersion': 2, 'profile': profile, 'target': paths.TARGETS[abi],
                  'defaultFeatures': not (core and abi in ['win32-ia32-msvc', 'win32-arm64-msvc']),
                  'sourceRecipe': chain.digest({file: snapshot[file] for file in snapshot if file in ['Cargo.toml', '.cargo/config.toml',
                      'rust-toolchain.toml', 'scripts/ci-build-rust-artifacts.sh'] or file.startswith('packages/plugin-tools/')}),
                  'toolchain': tomllib.loads(source['files']['rust-toolchain.toml'].decode())['toolchain']['channel']}
        value = {'owner': row['owner'], 'name': row['name'], 'version': row['version'], 'file': file, 'abi': abi,
                 'recipe': recipe, 'inputHash': closure(['cargo:' + producer], directory),
                 'abiCohort': 'node-api' if row['owner'] == 'create-farm' else cohort}
        result[row['owner']] = {**value, 'contentKey': chain.digest(value)}
    check(result, 'No native products for target')
    return result


def measured_context(source, capture, abi, image=None):
    check(isinstance(capture, dict), 'Measured capture must be an object')
    runtime = capture.get('runtimeCapture')
    check(isinstance(runtime, dict), 'Measured runtime must be an object')
    channel = tomllib.loads(source['files']['rust-toolchain.toml'].decode())['toolchain']['channel']
    check(runtime.get('abi') == abi and runtime.get('target') == paths.TARGETS[abi], 'Measured target differs')
    check(runtime.get('profile') in ['ci', 'release'], 'Unknown build role')
    check(not runtime.get('errors') and not runtime.get('unboundBuildVariables'), 'Incomplete measured compiler environment')
    active = runtime.get('activeToolchain')
    check(isinstance(active, str) and active.startswith(channel + '-'), 'Pinned compiler differs')
    host = runtime.get('runtime')
    check(isinstance(host, dict) and runtime.get('rustc') and runtime.get('cargo') and host.get('node') == 'v22.20.0', 'Missing formal tools')
    linker = runtime.get('linker')
    check(isinstance(linker, dict) and isinstance(linker.get('version'), str) and linker['version'].strip(), 'Unmeasured effective linker')
    # Environment is a compatibility dimension, never a content/source identity.
    if abi in ['linux-x64-gnu', 'linux-x64-musl']:
        container = capture.get('container') or {}
        check(container.get('buildExitCode') == 0 and container.get('timedOut') is False and container.get('cleanupConfirmed') is True, 'Container build did not complete')
        actual = container.get('containerImageId')
        check(isinstance(actual, str) and re.fullmatch('sha256:[a-f0-9]{64}', actual), 'Immutable image missing')
        if image is not None: check(actual == image, 'Formal image differs')
        platform = {'image': actual}
    else:
        check(runtime.get('platformSdk'), 'Missing platform SDK/toolset measurement')
        platform = {'sdk': runtime['platformSdk'], 'host': runtime['runtime']}
    return {'toolchain': channel, 'rustc': runtime['rustc'], 'cargo': runtime['cargo'],
            'linker': runtime['linker'], 'platform': platform, 'formalFlagsHash': runtime.get('formalFlagsHash')}


def create_catalog(source, capture, abi, run_id, attempt, root):
    check(capture.get('originBuildSHA') == source['sha'] and str(capture.get('runId')) == str(run_id) and str(capture.get('runAttempt')) == str(attempt), 'Producer capture identity differs')
    check(capture.get('profile') == 'release', 'Only main formal producers publish reusable catalogs')
    wanted = product_inputs(source, abi)
    context = measured_context(source, capture, abi)
    check(context['formalFlagsHash'], 'Effective formal recipe measurement missing')
    records, bodies = [], {}
    for row in capture['records']:
        expected = wanted[row['owner']]
        check(all(row[key] == expected[key] for key in ['name', 'file']), 'Product capture identity differs')
        file = workspace_path(root, expected['file'])
        check(file.is_file() and not file.is_symlink() and file.resolve().is_relative_to(Path(root).resolve()), 'Product path redirected')
        body = file.read_bytes()
        check(len(body) == row['bytes'] and hashlib.sha256(body).hexdigest() == row['sha256'], 'Captured bytes changed')
        check(native_matches(body, abi), 'Native binary target differs')
        member = hashlib.sha256(row['owner'].encode()).hexdigest() + Path(file).suffix
        bodies[member] = body
        records.append({**expected, 'member': member, 'bytes': len(body), 'sha256': row['sha256']})
    return {'schemaVersion': 2, 'kind': 'main-formal-native-products', 'sourceSHA': source['sha'], 'sourceTree': source['tree'],
            'runId': run_id, 'attempt': attempt, 'abi': abi, 'context': context, 'records': records}, bodies


def inspect_catalog(body, artifact, run, job, source, current, abi, desired_environment):
    check(run['path'] == WORKFLOW and run['head_branch'] == 'main' and run['event'] in ['push', 'workflow_dispatch'], 'Not a main formal producer')
    check(run['status'] == 'completed' and run['conclusion'] == 'success', 'Producer candidate validation did not succeed')
    check(run['repository']['id'] == run['head_repository']['id'] == desired_environment['repositoryId'], 'Fork or foreign producer')
    check(job['name'].endswith(('Build and Upload User Artifacts - ' + abi, 'Build and Upload Release Artifacts - ' + abi)), 'Unexpected producer job identity')
    check(job['run_id'] == run['id'] and job['status'] == 'completed' and job['conclusion'] == 'success', 'Native job failed')
    check(job.get('run_attempt') == run['run_attempt'], 'Native job attempt differs')
    check(artifact['workflow_run']['id'] == run['id'] and artifact['expired'] is False, 'Artifact expired or wrong run')
    check(artifact['name'] == f"formal-native-{abi}-{run['run_attempt']}", 'Wrong catalog attempt')
    execution.upload_binding(job, UPLOAD, artifact)
    check(0 < len(body) <= MAX and len(body) == artifact['size_in_bytes'] and artifact['digest'] == 'sha256:' + hashlib.sha256(body).hexdigest(), 'Catalog bytes differ')
    check(source['sha'] == run['head_sha'], 'Producer source differs')
    old, wanted = product_inputs(source, abi), product_inputs(current, abi)
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        entries = archive.infolist()
        check(0 < len(entries) <= 64 and sum(row.file_size for row in entries) <= MAX, 'Catalog bounds exceeded')
        paths.validate_member_set([row.filename for row in entries], abi)
        check(all(not row.is_dir() and stat.S_IFMT(row.external_attr >> 16) in [0, stat.S_IFREG] for row in entries), 'Catalog special member')
        check(archive.getinfo('catalog.json').file_size < 256 * 1024, 'Catalog metadata too large')
        manifest = json.loads(archive.read('catalog.json'))
        check(manifest['schemaVersion'] == 2 and manifest['kind'] == 'main-formal-native-products', 'Legacy catalog rejected')
        check(manifest['sourceSHA'] == source['sha'] and manifest['sourceTree'] == source['tree'] and manifest['runId'] == run['id'] and manifest['attempt'] == run['run_attempt'], 'Catalog provenance differs')
        check(manifest.get('abi') == abi, 'Catalog ABI differs')
        context = manifest['context']
        check(context['platform'] == desired_environment['platform'] and context['formalFlagsHash'] == desired_environment['formalFlagsHash'], 'Formal platform or flags differ')
        check(all(key in desired_environment and context.get(key) == desired_environment[key] for key in ['toolchain', 'rustc', 'cargo', 'linker']), 'Measured consumer compiler or linker differs or is missing')
        check(context['toolchain'] == next(iter(wanted.values()))['recipe']['toolchain'] and context['rustc'] and context['cargo'] and isinstance(context['linker'], dict) and isinstance(context['linker'].get('version'), str) and context['linker']['version'].strip(), 'Incomplete toolchain evidence')
        check(len({row['owner'] for row in manifest['records']}) == len(manifest['records']), 'Duplicate product')
        check({row.filename for row in entries} == {'catalog.json'} | {row['member'] for row in manifest['records']}, 'Unexpected catalog member')
        accepted = {}
        for row in manifest['records']:
            check(row['owner'] in old, 'Unknown producer owner')
            check(all(row[key] == value for key, value in old[row['owner']].items()), 'Producer fingerprint was not independently reproduced')
            product = archive.read(paths.safe_member(row['member']))
            check(len(product) == row['bytes'] and hashlib.sha256(product).hexdigest() == row['sha256'], 'Product digest differs')
            check(native_matches(product, abi), 'Native binary target differs')
            if row['owner'] in wanted and row['contentKey'] == wanted[row['owner']]['contentKey']:
                accepted[row['owner']] = {'record': row, 'bytes': product, 'originSHA': source['sha'], 'runId': run['id'], 'attempt': run['run_attempt'], 'artifactId': artifact['id']}
        return accepted


def resolve_products(transport, source, abi, desired, read_source=evidence.source_reader):
    wanted, accepted, reasons = product_inputs(source, abi), {}, []
    # Snapshot the candidate list once. Inspect the latest attempt of each run,
    # never retrieve an older successful attempt after a failed rerun.
    runs = transport.get('/actions/workflows/incremental-release.yaml/runs?per_page=20&branch=main')['workflow_runs']
    main_sha = transport.get('/commits/main')['sha']
    check(re.fullmatch('[a-f0-9]{40}', main_sha), 'Trusted main snapshot missing')
    for summary in runs:
        if len(accepted) == len(wanted): break
        try:
            run = transport.get('/actions/runs/' + str(summary['id']))
            if run['status'] != 'completed' or run['conclusion'] != 'success':
                reasons.append('latest-producer-attempt-not-successful'); continue
            check(run['run_attempt'] == summary['run_attempt'], 'Producer attempt moved during resolution')
            comparison = transport.get('/compare/' + run['head_sha'] + '...' + main_sha)
            check(comparison.get('status') in ['ahead', 'identical'] and comparison.get('merge_base_commit', {}).get('sha') == run['head_sha'], 'Producer is not an ancestor of fixed main snapshot')
            name = f"formal-native-{abi}-{run['run_attempt']}"
            rows = transport.get(f"/actions/runs/{run['id']}/artifacts?per_page=2&name={name}")['artifacts']
            if len(rows) != 1 or rows[0]['expired']: reasons.append('artifact-missing-or-expired'); continue
            artifact = transport.get('/actions/artifacts/' + str(rows[0]['id']))
            jobs = execution.jobs_for(transport, run)
            matching = [job for job in jobs if any(step.get('name') == UPLOAD for step in job.get('steps', []))
                        and abi in job['name']]
            check(len(matching) == 1, 'Ambiguous native producer job')
            products = inspect_catalog(transport.archive(artifact['id']), artifact, run, matching[0], read_source(run['head_sha']), source, abi, desired)
            for owner, value in products.items(): accepted.setdefault(owner, value)
        except PermissionError:
            reasons.append('artifact-access-denied-build-current'); break
        except Exception as error:
            reasons.append(str(error)[:300])
    return accepted, reasons


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('command', choices=['produce', 'resolve']); parser.add_argument('--abi', required=True); parser.add_argument('--output', required=True)
    parser.add_argument('--capture'); args = parser.parse_args()
    source = evidence.source_reader(chain.impact.git('rev-parse', 'HEAD').decode().strip())
    if args.command == 'produce':
        check(os.environ.get('GITHUB_SHA') == source['sha'] and os.environ.get('GITHUB_WORKFLOW_REF') == os.environ.get('GITHUB_REPOSITORY', '') + '/' + WORKFLOW + '@refs/heads/main', 'Producer workflow/source differs')
        check(os.environ.get('GITHUB_REF') == 'refs/heads/main' and os.environ.get('FARM_BUILD_PROFILE') == 'release', 'PR cannot produce trusted main catalog')
        capture = json.loads(Path(args.capture).read_text())
        try:
            measured_context(source, capture, args.abi)
            check(capture.get('records'), 'No selected native products for this target')
        except ValueError as error:
            print(json.dumps({'catalogAvailable': False, 'abi': args.abi, 'reason': str(error), 'consumerAction': 'build-current'}))
            if os.environ.get('GITHUB_OUTPUT'):
                with Path(os.environ['GITHUB_OUTPUT']).open('a') as stream: stream.write('catalog_available=false\n')
            return
        catalog, bodies = create_catalog(source, capture, args.abi, int(os.environ['GITHUB_RUN_ID']), int(os.environ['GITHUB_RUN_ATTEMPT']), Path.cwd())
        output = Path(args.output); output.mkdir(parents=True, exist_ok=False)
        (output / 'catalog.json').write_text(json.dumps(catalog, sort_keys=True))
        for file, body in bodies.items(): (output / file).write_bytes(body)
        if os.environ.get('GITHUB_OUTPUT'):
            with Path(os.environ['GITHUB_OUTPUT']).open('a') as stream: stream.write('catalog_available=true\n')
        return
    result = {'schemaVersion': 2, 'sourceSHA': source['sha'], 'sourceTree': source['tree'], 'abi': args.abi, 'reused': {}, 'acceptanceInherited': False}
    try:
        check(os.environ.get('FARM_BUILD_PROFILE') == 'ci', 'Release always builds current selected source')
        result['consumer'] = consumer_identity(source, args.abi, os.environ)
        nodes, issues = chain.repository_graph(source['files']); check(not issues, 'Invalid consumer graph')
        result['graphHash'] = chain.graph_contract(nodes, issues)['graphHash']
        result['products'] = product_inputs(source, args.abi)
        result['planHash'] = chain.digest({'consumer': result['consumer'], 'graphHash': result['graphHash'], 'products': result['products']})
        repository = os.environ['GITHUB_REPOSITORY']
        transport = evidence.transport_module.GitHubReadOnly(repository, os.environ.get('GH_TOKEN', ''), allow_archive=True)
        capture = {'runtimeCapture': json.loads(Path('.ci-native-proof', args.abi, 'runtime.json').read_text())}
        if args.abi in ['linux-x64-gnu', 'linux-x64-musl']:
            capture['container'] = json.loads(Path('.ci-native-cache', args.abi + '.json').read_text())
        desired = measured_context(source, capture, args.abi); result['context'] = desired
        desired['repositoryId'] = int(os.environ['GITHUB_REPOSITORY_ID'])
        accepted, result['reasons'] = resolve_products(transport, source, args.abi, desired)
        for owner, value in accepted.items():
            file = workspace_path(Path.cwd(), value['record']['file'], create_parents=True)
            check(not file.exists(), 'Reuse cannot overwrite existing native output')
            with file.open('xb') as stream: stream.write(value['bytes'])
            result['reused'][owner] = {key: item for key, item in value.items() if key != 'bytes'}
    except Exception as error:
        # Incomplete environment evidence is an ordinary build miss, not skip.
        result['reasons'] = [str(error)[:300]]
    output = Path(args.output); output.parent.mkdir(parents=True, exist_ok=True)
    body = (json.dumps(result, sort_keys=True) + '\n').encode(); output.write_bytes(body)
    if os.environ.get('GITHUB_OUTPUT'):
        with Path(os.environ['GITHUB_OUTPUT']).open('a') as stream: stream.write('resolution_hash=' + hashlib.sha256(body).hexdigest() + '\n')
    print(json.dumps({'reused': len(result['reused']), 'reasons': result.get('reasons', [])}))


if __name__ == '__main__': main()
