"""Loopback, real Git storage integration; no hosted identity or cloud guarantees."""
# cspell:words importlib sqlite NOSYSTEM mktree
from contextlib import closing
import hashlib
import http.server
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

spec = importlib.util.spec_from_file_location('publication', Path(__file__).with_name('incremental-publication-state.py'))
production = importlib.util.module_from_spec(spec)
spec.loader.exec_module(production)


def wait_for(predicate, timeout=20):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        if predicate():
            return
        time.sleep(0.02)
    raise AssertionError('Local integration synchronization timed out')


class GitAPI:
    """Test transport: real Git objects and atomic ref update, never a remote."""
    def __init__(self, directory, barrier=None):
        self.directory, self.barrier = Path(directory), barrier

    def git(self, *args, body=None, check=True):
        env = {**os.environ, 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': os.devnull,
               'GIT_AUTHOR_NAME': 'Local Fixture', 'GIT_AUTHOR_EMAIL': 'fixture@example.invalid',
               'GIT_COMMITTER_NAME': 'Local Fixture', 'GIT_COMMITTER_EMAIL': 'fixture@example.invalid'}
        p = subprocess.run(['git', '--git-dir', str(self.directory / 'state.git'), *args],
                           input=body, capture_output=True, env=env, timeout=10)
        if check and p.returncode:
            raise ValueError('Local Git atomic update rejected: ' + p.stderr.decode())
        return p

    def read_ref(self, reference):
        assert reference == 'heads/farm-release-state'
        p = self.git('rev-parse', '--verify', 'refs/' + reference, check=False)
        if p.returncode:
            assert p.returncode == 128
            return None
        return p.stdout.decode().strip()

    def read_state(self, commit):
        state = json.loads(self.git('show', commit + ':publication-state.json').stdout)
        if self.barrier:
            barrier, self.barrier = self.barrier, None
            (self.directory / ('ready-' + barrier)).write_text(commit)
            wait_for(lambda: (self.directory / 'go').exists())
        return state

    def create_state_commit(self, state, parents):
        blob = self.git('hash-object', '-w', '--stdin', body=json.dumps(state).encode()).stdout.decode().strip()
        tree = self.git('mktree', body=('100644 blob ' + blob + '\tpublication-state.json\n').encode()).stdout.decode().strip()
        return self.git('commit-tree', tree, *sum((['-p', p] for p in parents), []), body=b'local state\n').stdout.decode().strip()

    def create_ref(self, reference, commit):
        self.git('update-ref', 'refs/' + reference, commit, '0' * 40)

    def update_ref(self, reference, commit, force):
        assert force is False
        old = self.read_ref(reference)
        self.git('merge-base', '--is-ancestor', old, commit)
        self.git('update-ref', 'refs/' + reference, commit, old)


def serve(directory):
    directory = Path(directory)
    database = directory / 'registry.sqlite'
    with closing(sqlite3.connect(database)) as db, db:
        db.execute('CREATE TABLE IF NOT EXISTS packages (name TEXT PRIMARY KEY, body BLOB, writes INTEGER)')
    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_GET(self):
            with closing(sqlite3.connect(database)) as db, db:
                row = db.execute('SELECT body,writes FROM packages WHERE name=?', (self.path,)).fetchone()
            body = json.dumps(None if row is None else {'sha256': hashlib.sha256(row[0]).hexdigest(), 'writes': row[1]}).encode()
            self.send_response(200); self.end_headers(); self.wfile.write(body)

        def do_PUT(self):
            size = int(self.headers['Content-Length']); assert 0 < size < 1048576
            body = self.rfile.read(size)
            with closing(sqlite3.connect(database)) as db, db:
                db.execute('BEGIN IMMEDIATE')
                row = db.execute('SELECT body FROM packages WHERE name=?', (self.path,)).fetchone()
                if row is not None:
                    self.send_response(409); self.end_headers(); return
                db.execute('INSERT INTO packages VALUES (?,?,1)', (self.path, body))
            # Response only follows committed durable storage.
            self.send_response(201); self.end_headers()
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    (directory / 'endpoint').write_text('http://127.0.0.1:' + str(server.server_port))
    server.serve_forever()


class Registry:
    def __init__(self, directory, crash=False):
        self.url = (Path(directory) / 'endpoint').read_text()
        assert self.url.startswith('http://127.0.0.1:')
        self.crash = crash
        self.client = urllib.request.build_opener(urllib.request.ProxyHandler({}))

    def lookup(self, row):
        with self.client.open(self.url + '/' + row['name'], timeout=5) as response:
            data = json.load(response)
        return None if data is None else data['sha256']

    def publish(self, row, body):
        request = urllib.request.Request(self.url + '/' + row['name'], data=body, method='PUT')
        with self.client.open(request, timeout=5) as response:
            assert response.status == 201
        if self.crash:
            os._exit(77)  # Real abrupt writer exit after server commit, before local confirmation.


def worker(directory, role):
    d = Path(directory); manifest = json.loads((d / 'manifest.json').read_text())
    attempt = 1 if role in ['a', 'b', 'crash'] else 2
    api = GitAPI(d, role if role in ['a', 'b'] else None)
    transaction = production.Publication(production.GitRefStore(api), manifest,
                                        {'runId': 8 if role == 'b' else 7, 'attempt': attempt}, lambda value: value == manifest)
    # Authorization is an explicit fixture; not GitHub identity or attestation.
    try:
        if role in ['a', 'b']:
            transaction.acquire()
            (d / ('result-' + role)).write_text('acquired')
            wait_for(lambda: (d / 'finish').exists())
        else:
            if role == 'crash':
                transaction.acquire()
            else:
                transaction.recover(lambda old: (d / 'writer-terminal').read_text() == '77' and old == {'runId': 7, 'attempt': 1})
            transaction.upload(lambda row: (d / row['name']).read_bytes(), Registry(d, role == 'crash'))
            (d / ('result-' + role)).write_text('uploaded')
    except ValueError as error:
        (d / ('result-' + role)).write_text('rejected: ' + str(error))


def prepare(directory):
    d = Path(directory)
    subprocess.run(['git', 'init', '--bare', str(d / 'state.git')], check=True, capture_output=True)
    rows = []
    for name in ['one', 'two', 'three']:
        body = ('frozen original local fixture ' + name).encode(); (d / name).write_bytes(body)
        rows.append({'kind': 'npm', 'name': name, 'version': '2.0.0', 'archiveBytes': len(body), 'archiveSha256': hashlib.sha256(body).hexdigest()})
    manifest = {'chain': 'release', 'profile': 'release', 'sourceSHA': 'a' * 40, 'sourceTree': 'b' * 40,
                'planHash': 'c' * 64, 'baseline': None, 'packages': rows}
    (d / 'manifest.json').write_text(json.dumps(manifest))
    production.GitRefStore(GitAPI(d)).swap(None, {'baseline': None, 'active': None, 'completed': {}})
    return manifest


def start(directory, role):
    home = Path(directory) / 'empty-home'; home.mkdir(exist_ok=True)
    env = {'HOME': str(home), 'PATH': str(Path(sys.executable).parent) + os.pathsep + os.defpath, 'FARM_LOCAL_SIMULATION': 'true'}
    return subprocess.Popen([sys.executable, str(Path(__file__).resolve()), role, str(directory)],
                            env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)


def stop(process):
    if process.poll() is None:
        process.terminate()
        try: process.wait(timeout=5)
        except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=5)
    if process.stderr: process.stderr.close()


def concurrent_lock():
    with tempfile.TemporaryDirectory(prefix='farm-local-lock-') as directory:
        d = Path(directory); manifest = prepare(d); workers = [start(d, role) for role in ['a', 'b']]
        try:
            wait_for(lambda: all((d / ('ready-' + r)).exists() for r in ['a', 'b']))
            assert (d / 'ready-a').read_text() == (d / 'ready-b').read_text()
            (d / 'go').write_text('race')
            wait_for(lambda: all((d / ('result-' + r)).exists() and (d / ('result-' + r)).stat().st_size for r in ['a', 'b']))
            results = [(d / ('result-' + r)).read_text() for r in ['a', 'b']]
            assert results.count('acquired') == 1 and sum(s.startswith('rejected:') for s in results) == 1, results
            winner = workers[results.index('acquired')]; winner.kill(); winner.wait(timeout=5)
            store = production.GitRefStore(GitAPI(d))
            winner_run = 7 if results[0] == 'acquired' else 8
            recovery = production.Publication(store, manifest, {'runId': winner_run, 'attempt': 2}, lambda value: value == manifest)
            try: recovery.recover(lambda old: False)
            except ValueError: pass
            else: raise AssertionError('Unproven termination accepted')
            recovery.recover(lambda old: winner.returncode == -9 and old == {'runId': winner_run, 'attempt': 1})
            recovery.owned()
            assert store.read()[1]['baseline'] is None
            print(json.dumps({'localIntegration': 'concurrent-lock', 'competitors': 2, 'acquired': 1, 'rejected': 1, 'killedOwnerExit': winner.returncode, 'recoveryAttempt': 2, 'baselineAdvanced': False}), flush=True)
        finally:
            for process in workers: stop(process)


def durable_recovery():
    with tempfile.TemporaryDirectory(prefix='farm-local-recovery-') as directory:
        d = Path(directory); manifest = prepare(d); server = start(d, 'serve'); writer = None
        try:
            wait_for(lambda: (d / 'endpoint').exists()); writer = start(d, 'crash')
            writer_exit = writer.wait(timeout=20)
            assert writer_exit == 77
            (d / 'writer-terminal').write_text(str(writer.returncode))
            old = production.GitRefStore(GitAPI(d)).read()[1]
            assert old['active']['packages'] == {'npm:one': 'upload-started'}
            stop(server); (d / 'endpoint').unlink(); server = start(d, 'serve'); wait_for(lambda: (d / 'endpoint').exists())
            # Fresh process and restarted HTTP service read the same durable stores.
            stop(writer); writer = start(d, 'resume')
            writer_exit = writer.wait(timeout=20)
            assert writer_exit == 0
            assert (d / 'result-resume').read_text() == 'uploaded'
            with closing(sqlite3.connect(d / 'registry.sqlite')) as db, db:
                rows = db.execute('SELECT name,body,writes FROM packages ORDER BY name').fetchall()
                assert len(rows) == 3 and all(row[2] == 1 for row in rows)
                expected = {row['name']: row['archiveSha256'] for row in manifest['packages']}
                assert all(hashlib.sha256(body).hexdigest() == expected[name.lstrip('/')] for name, body, _ in rows)
            state = production.GitRefStore(GitAPI(d)).read()[1]
            assert state['active']['phase'] == 'public-acceptance' and state['baseline'] is None
            # Saved success cannot authorize swapped original bytes or registry conflicts.
            tx = production.Publication(production.GitRefStore(GitAPI(d)), manifest, {'runId': 7, 'attempt': 3}, lambda value: value == manifest)
            tx.recover(lambda old: old == {'runId': 7, 'attempt': 2} and writer.returncode == 0)
            tx.upload(lambda row: (d / row['name']).read_bytes(), Registry(d))
            original = (d / 'three').read_bytes(); (d / 'three').unlink()
            try: tx.upload(lambda row: (d / row['name']).read_bytes(), Registry(d))
            except FileNotFoundError: pass
            else: raise AssertionError('Missing archive accepted')
            (d / 'three').write_bytes(original)
            for tamper in ['archive', 'registry']:
                if tamper == 'archive': (d / 'two').write_bytes(b'tampered')
                else:
                    (d / 'two').write_bytes(b'frozen original local fixture two')
                    with closing(sqlite3.connect(d / 'registry.sqlite')) as db, db: db.execute('UPDATE packages SET body=? WHERE name=?', (b'conflicting', '/two'))
                try: tx.upload(lambda row: (d / row['name']).read_bytes(), Registry(d))
                except ValueError: pass
                else: raise AssertionError('Tampered bytes accepted')
            changed = dict(manifest, sourceSHA='d' * 40)
            other = production.Publication(production.GitRefStore(GitAPI(d)), changed, {'runId': 7, 'attempt': 4}, lambda value: value == changed)
            try: other.recover(lambda old: True)
            except ValueError: pass
            else: raise AssertionError('Changed frozen source accepted')
            with closing(sqlite3.connect(d / 'registry.sqlite')) as db:
                assert db.execute('SELECT SUM(writes) FROM packages').fetchone()[0] == 3
            print(json.dumps({'localIntegration': 'durable-recovery', 'crashExit': 77, 'serverRestarted': True, 'originalArchives': 3, 'successfulRegistryWrites': 3, 'repeatUploadNoop': True, 'missingArchiveRejected': True, 'changedArchiveRejected': True, 'registryConflictRejected': True, 'changedSourceRejected': True, 'baselineAdvanced': False}), flush=True)
        finally:
            if writer: stop(writer)
            stop(server)


if __name__ == '__main__':
    if sys.argv[1] == 'serve': serve(sys.argv[2])
    else: worker(sys.argv[2], sys.argv[1])
