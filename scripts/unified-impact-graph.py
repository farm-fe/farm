"""Canonical Cargo/npm/custom/global graph projection for all execution clients."""
import importlib.util
import json
from pathlib import Path
import sys

spec = importlib.util.spec_from_file_location('unified_chain', Path(__file__).with_name('incremental-chain-plan.py'))
chain = importlib.util.module_from_spec(spec)
spec.loader.exec_module(chain)


def project(snapshots):
    graphs, issues = [], []
    for snapshot in snapshots:
        files = {name: value.encode("utf-8") for name, value in snapshot.items()}
        # Small unit fixtures can model npm-only workspaces. Production callers
        # supply both committed workspace manifests and every Cargo manifest.
        if 'Cargo.toml' not in files:
            files['Cargo.toml'] = b'[workspace]\nmembers=[]\n'
        if 'pnpm-workspace.yaml' not in files:
            files['pnpm-workspace.yaml'] = b'packages:\n - packages/*\n - js-plugins/*\n - rust-plugins/*\n - examples/*\n'
        graph, errors = chain.repository_graph(files)
        graphs.append(graph)
        issues.extend(errors)
    graph = chain.union_graph(*graphs)
    return chain.graph_contract(graph, issues)



if __name__ == '__main__':
    if '--validate' in sys.argv:
        commit = chain.impact.git('rev-parse', 'HEAD').decode().strip()
        files, _ = chain.impact.snapshot(commit)
        graph, issues = chain.repository_graph(files)
        for row in chain.DIRECTORY_RULES['dependencies']:
            if 'npm:' + row['consumer'] not in graph or any('npm:' + value not in graph for value in row['dependencies']):
                raise ValueError('Custom dependency target absent from current workspace')
        print(json.dumps({'sourceSHA': commit, 'graphHash': chain.graph_contract(graph, issues)['graphHash'],
                          'issues': issues, 'nodes': len(graph)}))
    else:
        # JS sends UTF-8 JSON bytes. Windows stdin may use a legacy code page;
        # decode the wire format explicitly and reject invalid bytes, never drop them.
        payload = json.loads(sys.stdin.buffer.read().decode("utf-8"))
        print(json.dumps(project(payload['snapshots']), sort_keys=True))
