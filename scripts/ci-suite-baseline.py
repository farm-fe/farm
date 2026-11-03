"""Authorize current suite selection only after independent full CI acceptance."""
# cspell:words importlib argparse
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess

def load(name, file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value

acceptance=load('suite_baseline_acceptance','ci-execution-evidence.py')
evidence=acceptance.evidence

def ancestor(base, head):
    result=subprocess.run(['git','merge-base','--is-ancestor',base,head],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30)
    return result.returncode==0

def lookup(transport, current, environment, read_source=evidence.source_reader,is_ancestor=ancestor,
           suite_discovery=acceptance.runtime.discover_source_suites):
    repository=transport.get('')
    evidence.check(repository['id']==int(environment['GITHUB_REPOSITORY_ID']) and repository['default_branch']=='main','Trusted repository differs')
    trusted=read_source(transport.get('/branches/main')['commit']['sha'])
    evidence.check(current['policyHash']==trusted['policyHash'],'Current suite policy is not approved on main')
    runs=transport.get('/actions/workflows/ci.yaml/runs?per_page=3&status=completed&event=push&branch=main')['workflow_runs']
    for row in runs[:3]:
        try:
            run=transport.get('/actions/runs/'+str(row['id'])+'/attempts/'+str(row['run_attempt']))
            if run['conclusion']!='success':continue
            source=read_source(run['head_sha'])
            evidence.check(is_ancestor(source['sha'],trusted['sha']),'Suite baseline is not an ancestor of trusted main')
            jobs=acceptance.jobs_for(transport,run)
            aggregation,_=acceptance.actual_producer(transport,run,acceptance.PRODUCER,jobs)
            ledger,artifact=acceptance.fetch_json_artifact(transport,aggregation,'farm-ci-execution-baseline-'+str(run['id'])+'-'+str(aggregation['run_attempt']))
            proof=acceptance.verify_execution_ledger(transport,run,repository,ledger,artifact,jobs,source,trusted,acceptance.runtime.ci_required_jobs(),suite_discovery=suite_discovery)
        except (ValueError,KeyError,OSError,UnicodeError,subprocess.SubprocessError):
            continue
        evidence.check(transport.get('/branches/main')['commit']['sha']==trusted['sha'],'Trusted baseline moved')
        return {'kind':'independently-verified-ci-suite-baseline','sourceSHA':current['sha'],'sourceTree':current['tree'],
                'baselineSHA':source['sha'],'baselineTree':source['tree'],'baselineEvidence':proof,
                'repositoryId':repository['id'],'runId':int(environment['GITHUB_RUN_ID']),'attempt':int(environment['GITHUB_RUN_ATTEMPT'])}
    raise ValueError('No complete trusted main suite baseline')

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--output',required=True);args=parser.parse_args()
    commitment=''
    try:
        evidence.check(os.environ.get('GITHUB_EVENT_NAME') in ['pull_request','push'],'Unsupported suite event')
        event=json.loads(Path(os.environ['GITHUB_EVENT_PATH']).read_text(encoding='utf-8'))
        if os.environ['GITHUB_EVENT_NAME']=='pull_request':
            evidence.check(event['pull_request']['head']['repo']['full_name']==os.environ['GITHUB_REPOSITORY'],'Fork runs full suites')
        else:evidence.check(os.environ.get('GITHUB_REF')=='refs/heads/main','Only main push suites are supported')
        transport=evidence.transport_module.GitHubReadOnly(os.environ['GITHUB_REPOSITORY'],os.environ['GH_TOKEN'],allow_archive=True)
        source=evidence.source_reader(os.environ['GITHUB_SHA'])
        identity=lookup(transport,source,os.environ)
        result=subprocess.run(['node','scripts/ci-suite-selection.mjs'],input=json.dumps(identity).encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True,timeout=90)
        evidence.check(len(result.stdout)<=2*1024**2,'Suite selection exceeds bound')
        output=Path(args.output);output.parent.mkdir(parents=True,exist_ok=True)
        with output.open('xb') as stream:stream.write(result.stdout)
        commitment=hashlib.sha256(result.stdout).hexdigest()
    except (ValueError,KeyError,OSError,UnicodeError,subprocess.SubprocessError):
        print('Suite baseline unavailable; execute full current suites')
    if os.environ.get('GITHUB_OUTPUT'):
        with Path(os.environ['GITHUB_OUTPUT']).open('a',encoding='utf-8') as stream:stream.write('selection_hash='+commitment+'\n')

if __name__=='__main__':main()
