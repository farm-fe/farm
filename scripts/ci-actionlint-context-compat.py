"""Narrow actionlint 1.7.12 compatibility for documented GitHub.com job fields.

GitHub documents job.workflow_sha/ref for reusable workflow self-checkout:
https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#job-context
This does not erase diagnostics: the report records each documented compatibility
case, and any other diagnostic retains a failure exit status.
"""
import argparse
import json
from pathlib import Path
import subprocess

WORKFLOW = '.github/workflows/ci-provenance-signer.yaml'
OBJECT = '{check_run_id: number; container: {id: string; network: string}; services: {string => {id: string; network: string; ports: {string => string}}}; status: string}'
ALLOWED = {
    'ref: ${{ job.workflow_sha }}':'workflow_sha',
    'SIGNER_WORKFLOW_SHA: ${{ job.workflow_sha }}':'workflow_sha',
    'SIGNER_WORKFLOW_REF: ${{ job.workflow_ref }}':'workflow_ref',
}


def classify(diagnostics, source):
    if not isinstance(diagnostics, list):
        raise ValueError('Malformed actionlint diagnostics')
    lines = source.splitlines(); known, remaining, seen = [], [], set()
    for row in diagnostics:
        line = row.get('line') if isinstance(row, dict) else None
        statement = lines[line-1].strip() if type(line) is int and 0 < line <= len(lines) else ''
        field = ALLOWED.get(statement)
        expected = 'property "' + str(field) + '" is not defined in object type ' + OBJECT
        column = lines[line-1].find('job.' + str(field)) + 1 if field else -1
        if (isinstance(row, dict) and row.get('filepath') == WORKFLOW and row.get('kind') == 'expression'
                and field and row.get('message') == expected and row.get('column') == column
                and statement not in seen):
            seen.add(statement); known.append(row)
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
    report = classify(json.loads(result.stdout or '[]'), Path(WORKFLOW).read_text())
    if result.returncode and not report['documentedContextCompatibility'] and not report['errors']:
        raise ValueError('actionlint failed without diagnostics')
    print(json.dumps(report, indent=2))
    raise SystemExit(1 if report['errors'] else 0)


if __name__ == '__main__':
    main()
