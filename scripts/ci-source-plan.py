"""Directory-identity source plans with explicit conservative historical coverage."""
# cspell:words importlib
import importlib.util
import hashlib
import json
from pathlib import Path


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


impact = load('source_plan_impact', 'ci-impact-plan.py')
HISTORICAL = 'all-committed-inputs-v1'


def committed_graph(files, hashes, *, historical_allowed):
    if set(files) != set(hashes):
        raise ValueError('Complete committed input records required')
    for name, body in files.items():
        record = hashes[name]
        if record.get('mode') not in ['100644', '100755'] or record.get('sha256') != hashlib.sha256(body).hexdigest():
            raise ValueError('Committed source record differs from bytes or mode')
    shared = load('source_plan_shared', 'incremental-chain-plan.py')
    path = 'scripts/directory-impact-rules.json'
    historical = path not in files
    if historical and not historical_allowed:
        raise ValueError('Tested source requires committed directory rules')
    # This is an explicit analysis policy, never fabricated historical rules.
    rules = ({'version': 1, 'global': [], 'suiteGlobal': [], 'dependencies': []}
             if historical else json.loads(files[path]))
    shared.DIRECTORY_RULES = shared.validate_directory_rules(rules)
    nodes, issues = shared.repository_graph(files)
    if issues:
        raise ValueError('Incomplete or ambiguous source graph: ' + ','.join(issues))
    public = set()
    for node in nodes.values():
        if node['kind'] == 'input':
            continue
        package = node['data'].get('package', {}) if node['kind'] == 'cargo' else node['data']
        published = ((node['kind'] == 'npm' and node['directory'].split('/')[0] in ['packages', 'js-plugins', 'rust-plugins'] and not package.get('private'))
                     or (node['kind'] == 'cargo' and package.get('publish') is not False and package.get('publish') != []))
        name = node['kind'] + ':' + node['name']
        if published and name in public:
            raise ValueError('Ambiguous publication package identity')
        if published:
            public.add(name)
    source_digest = impact.digest(hashes)
    if historical:
        key = 'input:historical-all-committed'
        nodes[key] = {'kind': 'input', 'directory': '@inputs/' + key, 'name': key,
                      'manifest': None, 'version': None, 'data': {}, 'paths': [],
                      'coverage': HISTORICAL, 'sourceRecordsDigest': source_digest,
                      'productDeps': set(), 'preparationDeps': set(), 'testDeps': set()}
        for other, node in nodes.items():
            if other != key:
                for field in ['productDeps', 'preparationDeps', 'testDeps']:
                    node[field].add(key)
    metadata = {'schemaVersion': 2, 'rulesSource': 'current-conservative-policy' if historical else 'committed-file',
                'policy': HISTORICAL if historical else 'committed-directory-rules-v1',
                'rulesPath': None if historical else path, 'rulesDigest': None if historical else impact.digest(rules),
                'sourceRecordsDigest': source_digest, 'sourceFileCount': len(files),
                'graphHash': shared.graph_contract(nodes, [])['graphHash']}
    return nodes, metadata


def fingerprints(nodes, metadata):
    # Source proof intentionally binds ALL tracked inputs, not a reusable binary
    # key. This conservative superset includes path, mode and content changes.
    result = {}
    for key in nodes:
        def closure(fields):
            pending, seen = [key], set()
            while pending:
                item = pending.pop()
                if item in seen:
                    continue
                seen.add(item)
                for field in fields:
                    pending.extend(nodes[item][field])
            return sorted(seen)
        product = closure(['productDeps', 'preparationDeps'])
        tests = closure(['productDeps', 'preparationDeps', 'testDeps'])
        identity = {'ownerNode': key, **metadata}
        result[key] = {'product': impact.digest({**identity, 'closure': product}),
                       'tests': impact.digest({**identity, 'closure': tests}),
                       'localTestSource': impact.digest(identity), 'closureHash': impact.digest(product),
                       'testClosureHash': impact.digest(tests), 'acceptanceKeyComplete': False}
    return result


def make_plan(base, head, tested, scope='e2e', release_candidate=False):
    for sha in [base, head, tested]:
        impact.revision(sha)
    bases = impact.git('merge-base', '--all', base, head).decode().split()
    if len(bases) != 1:
        raise ValueError('Ambiguous merge base')
    cache, snapshots, metadata, graphs = {}, {}, {}, {}
    for label, sha in [('mergeBase', bases[0]), ('base', base), ('head', head), ('tested', tested)]:
        if sha not in cache:
            cache[sha] = impact.snapshot(sha)
        files, hashes = cache[sha]
        nodes, proof = committed_graph(files, hashes, historical_allowed=label != 'tested')
        snapshots[label] = files; graphs[label] = nodes
        metadata[label] = {**proof, 'commit': sha, 'tree': impact.git('rev-parse', sha + '^{tree}').decode().strip()}
    by_snapshot = {id(snapshots[label]): graphs[label] for label in snapshots}
    def graph_for(files):
        # The old proposed-DAG projector has two edge classes. Over-approximate
        # preparation as product input; the authoritative graph retains all three.
        return {key: {**node, 'productDeps': node['productDeps'] | node['preparationDeps']}
                for key, node in by_snapshot[id(files)].items()}
    changes = impact.changed_paths(bases[0], head)
    tested_changes = impact.changed_paths(head, tested)
    plan = impact.plan_changes(snapshots['mergeBase'], snapshots['head'], changes + tested_changes,
                               extra_snapshots=[snapshots['base'], snapshots['tested']], graph_factory=graph_for)
    if any(value['policy'] == HISTORICAL for value in metadata.values()):
        plan.update(fullRequired=True, proposedSuites=impact.ALL, proposedExamples=[],
                    proposedExampleSelection={'mode': 'all', 'directories': [], 'executionEnabled': False})
        plan['reasons'].append({'reason': HISTORICAL, 'scope': 'all-committed-inputs; no historical custom edges inferred'})
    plan['proposedDag'] = impact.proposed_dag(plan['proposedSuites'], plan['proposedExamples'])
    plan['proposedDag']['exampleSelection'] = plan['proposedExampleSelection']
    plan.update(schemaVersion=impact.SCHEMA, policy=impact.POLICY, mode='shadow', skipAuthorized=False,
                reuseAuthorized=False, sourceGraphVersion=2, sourceInputProofComplete=True,
                sourceInputProofScope='committed-input-coverage-only', sourceGraphs=metadata,
                baseSHA=base, mergeBaseSHA=bases[0], headSHA=head, testedSHA=tested,
                testedTree=metadata['tested']['tree'], baseTree=metadata['base']['tree'], headTree=metadata['head']['tree'],
                changes=changes, testedMergeChanges=tested_changes,
                fingerprints={label: fingerprints(graphs[label], metadata[label]) for label in graphs},
                executedSuites=impact.ALL, profiles=['ci', 'ci-test', 'release'],
                runId=impact.os.environ.get('GITHUB_RUN_ID'), runAttempt=impact.os.environ.get('GITHUB_RUN_ATTEMPT'),
                repository=impact.os.environ.get('GITHUB_REPOSITORY'), producerTrust='not-enabled',
                executionScope=scope, executionCells=impact.execution_cells(scope, release_candidate))
    plan['planHash'] = impact.digest(plan)
    return plan


def verify_main_plan(plan, scope):
    if (plan.get('sourceGraphVersion') != 2 or plan.get('sourceInputProofComplete') is not True
            or plan.get('sourceInputProofScope') != 'committed-input-coverage-only'
            or plan.get('planningFallback') or plan.get('unavailable')):
        raise ValueError('Complete independently reproducible main source plan required')
    if plan.get('headSHA') != plan.get('testedSHA'):
        raise ValueError('Main source plan must test its exact head commit')
    expected = make_plan(plan['baseSHA'], plan['headSHA'], plan['testedSHA'], scope)
    for field in ['runId', 'runAttempt', 'repository']:
        expected[field] = plan.get(field)  # already authenticated by functional verifier
    expected['planHash'] = impact.digest({key: value for key, value in expected.items() if key != 'planHash'})
    if expected != plan:
        raise ValueError('Main source plan differs from fixed Git objects and analysis policy')
