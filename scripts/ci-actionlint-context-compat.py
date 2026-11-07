"""Narrow actionlint 1.7.12 compatibility for documented GitHub.com job fields.

GitHub documents job.workflow_sha/ref for reusable workflow self-checkout:
https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#job-context
This does not erase diagnostics: the report records each documented compatibility
case, and any other diagnostic retains a failure exit status.
"""
# cspell:words shellcheck
import argparse
import json
from pathlib import Path
import subprocess

WORKFLOW = '.github/workflows/ci-provenance-signer.yaml'
QUEUE_WORKFLOWS = {'.github/workflows/cargo-source-release.yaml', '.github/workflows/incremental-release.yaml'}
WORKFLOWS = {WORKFLOW, '.github/workflows/release-provenance-signer.yaml'}
OBJECT = '{check_run_id: number; container: {id: string; network: string}; services: {string => {id: string; network: string; ports: {string => string}}}; status: string}'
ALLOWED = {
    'ref: ${{ job.workflow_sha }}':'workflow_sha',
    'SIGNER_WORKFLOW_SHA: ${{ job.workflow_sha }}':'workflow_sha',
    'SIGNER_WORKFLOW_REF: ${{ job.workflow_ref }}':'workflow_ref',
}


def classify(diagnostics, source):
    if not isinstance(diagnostics, list):
        raise ValueError('Malformed actionlint diagnostics')
    sources = source if isinstance(source, dict) else {WORKFLOW:source}
    known, remaining, seen = [], [], set()
    for row in diagnostics:
        filename = row.get('filepath') if isinstance(row, dict) else None
        lines = sources.get(filename, '').splitlines()
        line = row.get('line') if isinstance(row, dict) else None
        statement = lines[line-1].strip() if type(line) is int and 0 < line <= len(lines) else ''
        # github.com supports queue:max; preserve the exact serial release policy.
        # https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency
        queue_message = 'unexpected key "queue" for "concurrency" section. expected one of "cancel-in-progress", "group"'
        queue_policy = (type(line) is int and 4 <= line <= len(lines) and lines[line-4] == 'concurrency:' and lines[line-3].startswith('  group: ') and lines[line-2] == '  cancel-in-progress: false' and lines[line-1] == '  queue: max')
        if (isinstance(row, dict) and filename in QUEUE_WORKFLOWS and queue_policy
                and statement == 'queue: max' and lines[line-1] == '  queue: max'
                and row.get('kind') == 'syntax-check' and row.get('message') == queue_message
                and row.get('column') == 3 and (filename, statement) not in seen):
            seen.add((filename, statement)); known.append(row); continue
        field = ALLOWED.get(statement)
        expected = 'property "' + str(field) + '" is not defined in object type ' + OBJECT
        column = lines[line-1].find('job.' + str(field)) + 1 if field else -1
        if (isinstance(row, dict) and filename in WORKFLOWS and row.get('kind') == 'expression'
                and field and row.get('message') == expected and row.get('column') == column
                and (filename, statement) not in seen):
            seen.add((filename, statement)); known.append(row)
        else:
            remaining.append(row)
    return {'documentedContextCompatibility':known, 'errors':remaining}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--actionlint', required=True)
    parser.add_argument('workflows', nargs='+')
    args = parser.parse_args()
    version = subprocess.run([args.actionlint, '-version'], capture_output=True, text=True, check=True)
    if not version.stdout.startswith('1.7.12\n'):
        raise ValueError('This compatibility policy requires reviewed actionlint 1.7.12')
    result = subprocess.run([args.actionlint, '-shellcheck=', '-format', '{{json .}}', *args.workflows], capture_output=True, text=True)
    if result.returncode not in [0,1] or result.stderr:
        raise ValueError('actionlint execution failed: ' + result.stderr[:2000])
    report = classify(json.loads(result.stdout or '[]'), {name:Path(name).read_text() for name in WORKFLOWS | QUEUE_WORKFLOWS if Path(name).is_file()})
    if result.returncode and not report['documentedContextCompatibility'] and not report['errors']:
        raise ValueError('actionlint failed without diagnostics')
    print(json.dumps(report, indent=2))
    raise SystemExit(1 if report['errors'] else 0)


if __name__ == '__main__':
    main()
