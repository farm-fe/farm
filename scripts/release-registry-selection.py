"""Exact registry version selection; immutable source and frozen observation inputs."""
# cspell:words importlib urllib npmjs tomllib SIGALRM ITIMER setitimer getitimer
from contextlib import contextmanager, nullcontext
import base64
import binascii
import importlib.util
import math
import json
from pathlib import Path
import re
import signal
import subprocess
import time
import urllib.error
import urllib.parse


def load(name, file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

chain=load('registry_selection_chain','incremental-chain-plan.py')
registry_module=load('registry_selection_reader','incremental-registry-evidence.py')


def check(value,message):
    if not value:raise ValueError(message)


def records(source,kind):
    nodes,issues=chain.repository_graph(source['files'])
    check(not issues,'Registry selection requires complete dependency graph')
    family=chain.independent_family(source['files'],nodes)
    actual={key:row for key,row in family.items() if row['kind']==kind}
    if kind=='npm':
        # npm payload assembly retains its complete reviewed candidate inventory.
        # Cargo versions are independently declared by the fixed SDK source graph;
        # release-plz updates must not depend on stale npm candidate metadata.
        declared=chain.family_records(json.loads(source['files']['docs/releases/v2-candidate.json']))
        fields=['kind','name','version','manifestPath','owner','platform']
        expected={key:row for key,row in declared.items() if row['kind']==kind}
        check(set(actual)==set(expected) and all(all(row.get(field)==expected[key].get(field) for field in fields) for key,row in actual.items()),'Declared full version family differs from source')
    chain.validate_family_sources(actual,source['files'])
    return actual


POLICY = 'registry-exact-version-v2'
METADATA_FIELDS = ['dependencies', 'optionalDependencies', 'peerDependencies', 'peerDependenciesMeta']


def safe_tarball_url(value):
    check(isinstance(value, str) and len(value) <= 2048, 'Registry archive URL invalid')
    parsed = urllib.parse.urlsplit(value)
    check(parsed.scheme == 'https' and parsed.hostname == 'registry.npmjs.org'
          and parsed.port in [None, 443] and not parsed.username and not parsed.password
          and not parsed.query and not parsed.fragment and parsed.path.startswith('/')
          and not any(ord(char) < 33 or ord(char) == 127 for char in value), 'Registry archive origin differs')
    return value


def unknown(row, error):
    # Never persist response bodies, URLs, exception messages or credentials.
    if isinstance(error, urllib.error.HTTPError):
        code = 'registry-http-' + str(error.code) if type(error.code) is int and 100 <= error.code <= 599 else 'registry-http-error'
    elif isinstance(error, (TimeoutError, ConnectionError, urllib.error.URLError)):
        code = 'registry-unavailable'
    elif isinstance(error, (ValueError, KeyError, TypeError, AttributeError)):
        code = 'registry-invalid-evidence'
    else:
        code = 'registry-read-failed'
    return {'status': 'unknown', 'name': row['name'], 'version': row['version'], 'reason': code}


class ExactRegistryReadOnly(registry_module.RegistryReadOnly):
    def archive(self, url):
        before = self.total_bytes
        result = super().archive(url)
        return {**result, 'bytes': self.total_bytes - before}

    def json(self, url):
        with self.budget(30):
            with self.open(url) as response:
                check(response.getcode() == 200, 'Exact registry metadata requires HTTP 200')
                body = response.read(2 * 1024 * 1024 + 1)
        check(len(body) <= 2 * 1024 * 1024, 'Registry metadata exceeded bound')
        return json.loads(body.decode('utf-8'))


class Registry:
    def __init__(self, reader=None):
        self.reader=reader or ExactRegistryReadOnly()

    def lookup(self,row, *, verify_archive=False):
        kind,name,version=row['kind'],row['name'],row['version']
        check(kind in ['npm','cargo'],'Unknown publication registry')
        check(re.fullmatch(r'\d+\.\d+\.\d+',version),'Exact stable version required')
        check(re.fullmatch(r'(?:@[a-z0-9._-]+/)?[a-zA-Z0-9._-]+',name),'Unsafe registry package name')
        url=('https://registry.npmjs.org/'+urllib.parse.quote(name,safe='')+'/'+version if kind=='npm' else 'https://crates.io/api/v1/crates/'+name+'/'+version)
        try:data=self.reader.json(url)
        except urllib.error.HTTPError as error:
            if error.code==404:return {'status':'missing','name':name,'version':version}
            raise
        if kind=='npm':
            check(data.get('name')==name and data.get('version')==version,'Registry metadata identity differs')
            dist=data['dist'];archive=dist['tarball']
            safe_tarball_url(archive)
            integrity = dist.get('integrity', '')
            check(isinstance(integrity, str), 'Registry integrity invalid')
            hashes = [value for value in integrity.split() if valid_sha512(value)]
            if not verify_archive:
                check(bool(hashes), 'Registry SHA512 integrity missing')
                metadata = {key: data.get(key, {}) for key in METADATA_FIELDS}
                validate_metadata(metadata)
                return {'status': 'published', 'name': name, 'version': version,
                        'metadata': metadata, 'integrity': sorted(set(hashes))[0],
                        'tarballURL': archive, 'evidenceKind': 'registry-metadata'}
            actual=self.reader.archive(archive)
            check(('sha512-'+actual['sha512'] in integrity.split()) if integrity else dist.get('shasum')==actual['sha1'],'Registry archive integrity differs')
            metadata={key:data.get(key,{}) for key in METADATA_FIELDS}
            validate_metadata(metadata)
        else:
            item=data['version'];check(item.get('crate')==name and item.get('num')==version and item.get('yanked') is False,'Cargo version identity or yanked state differs')
            actual=self.reader.archive('https://static.crates.io/crates/'+name+'/'+name+'-'+version+'.crate')
            check(item['checksum']==actual['sha256'],'Cargo archive checksum differs');metadata={}
        return {'status':'published','name':name,'version':version,'archiveSha256':actual['sha256'],'archiveBytes':actual['bytes'],'evidenceKind':'registry-archive','metadata':metadata,**({'integrity':'sha512-'+actual['sha512'],'tarballURL':archive} if kind=='npm' else {})}


def valid_sha512(value):
    if not isinstance(value, str) or not re.fullmatch(r'sha512-[A-Za-z0-9+/]+={0,2}', value):
        return False
    try:
        return len(base64.b64decode(value[7:], validate=True)) == 64
    except (ValueError, binascii.Error):
        return False


def validate_metadata(metadata):
    check(isinstance(metadata, dict), 'Registry dependency metadata invalid')
    for section in METADATA_FIELDS:
        values = metadata.get(section, {})
        check(isinstance(values, dict), 'Registry dependency metadata invalid')
        for name, value in values.items():
            check(isinstance(name, str) and bool(name), 'Registry dependency identity invalid')
            if section == 'peerDependenciesMeta':
                check(isinstance(value, dict) and ('optional' not in value or type(value['optional']) is bool), 'Registry peer metadata invalid')
            else:
                check(isinstance(value, str), 'Registry dependency range invalid')


def validate_observed(row, observed, schema_version):
    check(isinstance(observed, dict) and observed.get('name') == row['name'] and observed.get('version') == row['version'], 'Registry observation version differs')
    check(observed.get('status') in (['missing', 'published', 'unknown'] if schema_version == 2 else ['missing', 'published']), 'Registry error is not absence')
    if observed['status'] == 'published':
        validate_metadata(observed.get('metadata'))
        if schema_version == 2 and row['kind'] == 'npm':
            check(observed.get('evidenceKind') in ['registry-metadata', 'registry-archive'], 'Published evidence kind missing')
            check(valid_sha512(observed.get('integrity')), 'Published version integrity incomplete')
            safe_tarball_url(observed.get('tarballURL'))
            if observed['evidenceKind'] == 'registry-metadata':
                check('archiveSha256' not in observed and 'archiveBytes' not in observed
                      and observed.get('registryBytesVerified') is not True, 'Metadata-only evidence cannot claim archive verification')
            else:
                check(isinstance(observed.get('archiveSha256'), str) and re.fullmatch('[a-f0-9]{64}', observed['archiveSha256']), 'Published version evidence incomplete')
                check(type(observed.get('archiveBytes')) is int and 0 < observed['archiveBytes'] <= 100 * 1024 * 1024, 'Published version size incomplete')
        else:
            check(isinstance(observed.get('archiveSha256'), str) and re.fullmatch('[a-f0-9]{64}', observed['archiveSha256']), 'Published version evidence incomplete')
    elif observed['status'] == 'unknown':
        reason = observed.get('reason', '')
        check(isinstance(reason, str) and (re.fullmatch(r'registry-(?:http-[1-5][0-9]{2}|http-error|unavailable|invalid-evidence|read-failed)', reason)
              or reason in ['prior-submission-unconfirmed', 'prior-submission-history-unavailable', 'prior-submission-history-incomplete']), 'Registry unknown reason invalid')
        if 'priorSubmission' in observed:
            prior = observed['priorSubmission']
            check(isinstance(prior, dict) and set(prior) == {'runId', 'attempt', 'sourceSHA', 'artifactId', 'artifactDigest', 'name', 'version'}, 'Prior submission evidence fields differ')
            check(all(type(prior.get(key)) is int and prior[key] > 0 for key in ['runId', 'attempt', 'artifactId'])
                  and prior['attempt'] <= 20 and prior['name'] == row['name'] and prior['version'] == row['version'], 'Prior submission evidence identity differs')
            chain.exact(prior['sourceSHA'], 40)
            check(isinstance(prior['artifactDigest'], str) and re.fullmatch(r'(?:sha256:)?[a-f0-9]{64}', prior['artifactDigest']), 'Prior submission artifact digest invalid')


class RegistryLookupDeadline(BaseException):
    """Only this lookup's alarm is converted into a package-local unknown."""


class RegistryCollectionDeadline(BaseException):
    """The collection alarm preserves every already-recorded observation."""


class ExternalRegistryAlarm(BaseException):
    def __init__(self, original):
        self.original = original


@contextmanager
def registry_alarm(seconds, failure):
    check(isinstance(seconds, (int, float)) and not isinstance(seconds, bool)
          and math.isfinite(seconds) and 0 < seconds <= 240, 'Invalid registry observation deadline')
    check(hasattr(signal, 'setitimer'), 'Registry observation deadline requires the Linux planner')
    previous_handler = signal.getsignal(signal.SIGALRM)
    previous_delay, previous_interval = signal.getitimer(signal.ITIMER_REAL)
    started = time.monotonic()
    own = not previous_delay or seconds < previous_delay
    def interrupt(_number, _frame):
        raise failure()
    if own:
        signal.signal(signal.SIGALRM, interrupt)
        signal.setitimer(signal.ITIMER_REAL, seconds)
    try:
        yield
    except BaseException as error:
        # Preserve a shorter outer alarm's exception, even if it derives from
        # Exception and would otherwise look like an ordinary registry error.
        if (not own and not isinstance(error, (RegistryLookupDeadline, RegistryCollectionDeadline, ExternalRegistryAlarm))
                and time.monotonic() - started >= previous_delay):
            raise ExternalRegistryAlarm(error) from error
        raise
    finally:
        if own:
            signal.setitimer(signal.ITIMER_REAL, 0)
            signal.signal(signal.SIGALRM, previous_handler)
            if previous_delay:
                signal.setitimer(signal.ITIMER_REAL, max(0.001, previous_delay - (time.monotonic() - started)), previous_interval)


def collect(source, kind, lookup, *, schema_version=None, clock=time.monotonic, seconds=240, per_package_seconds=30):
    if schema_version is None:
        schema_version = 2 if kind == 'npm' else 1
    family = records(source, kind)
    rows = {}
    deadline = clock() + seconds
    reader = getattr(getattr(lookup, '__self__', None), 'reader', None)
    original_limit = getattr(reader, 'total_byte_limit', None)
    bounded_reader = (schema_version == 2 and type(original_limit) is int
                      and type(getattr(reader, 'total_bytes', None)) is int and callable(getattr(reader, 'budget', None)))
    # Reserve a separate 100 MiB archive plus one bounded over-read chunk per
    # package. A malformed/oversized earlier archive cannot consume a later
    # package's allowance. The reader still enforces its 100 MiB archive cap.
    batch_limit = min(100 * 1024 ** 3, max(original_limit, len(family) * 101 * 1024 ** 2)) if bounded_reader else None
    try:
        try:
            batch_alarm = registry_alarm(seconds, RegistryCollectionDeadline) if schema_version == 2 else nullcontext()
            with batch_alarm:
                for key, row in sorted(family.items()):
                    try:
                        remaining = deadline - clock()
                        if schema_version == 2 and remaining <= 0:
                            raise TimeoutError('Registry observation deadline exceeded')
                        if bounded_reader:
                            reader.total_byte_limit = min(batch_limit, reader.total_bytes + 100 * 1024 ** 2)
                        budget = (reader.budget(max(1, math.ceil(min(per_package_seconds, remaining))))
                                  if bounded_reader else nullcontext())
                        lookup_alarm = (registry_alarm(min(per_package_seconds, remaining), RegistryLookupDeadline)
                                        if schema_version == 2 else nullcontext())
                        with lookup_alarm, budget:
                            observed = lookup(row)
                        validate_observed(row, observed, schema_version)
                    except RegistryLookupDeadline:
                        observed = unknown(row, TimeoutError())
                    except Exception as error:
                        if schema_version == 1:
                            raise
                        observed = unknown(row, error)
                    rows[key] = observed
        except RegistryCollectionDeadline:
            for key, row in sorted(family.items()):
                if key not in rows:
                    rows[key] = unknown(row, TimeoutError())
    except ExternalRegistryAlarm as error:
        raise error.original
    finally:
        if bounded_reader:
            reader.total_byte_limit = original_limit
    return {'schemaVersion': schema_version, 'kind': 'exact-registry-version-observation', 'sourceSHA': source['sha'], 'sourceTree': source['tree'], 'registryKind': kind, 'packages': rows}


def validate(source,kind,snapshot):
    family=records(source,kind)
    check(type(snapshot.get('schemaVersion')) is int and snapshot['schemaVersion'] in [1, 2] and snapshot.get('kind')=='exact-registry-version-observation' and snapshot.get('registryKind')==kind,'Registry observation policy differs')
    if snapshot['schemaVersion'] == 2:
        check('stateBaseline' not in snapshot, 'Stateless registry observation cannot bind historical state')
    elif snapshot.get('stateBaseline') is not None:
        chain.exact(snapshot['stateBaseline'],64)
    check(snapshot.get('sourceSHA')==source['sha'] and snapshot.get('sourceTree')==source['tree'],'Registry observation source differs')
    check(isinstance(snapshot.get('packages'), dict) and set(snapshot['packages'])==set(family),'Registry observation full family differs')
    for key,row in family.items():
        validate_observed(row, snapshot['packages'][key], snapshot['schemaVersion'])
    return family


def verify_retained(source,snapshot,lookup, *, isolate=False):
    family=validate(source,snapshot['registryKind'],snapshot)
    failures = {}
    for key,row in family.items():
        previous=snapshot['packages'][key]
        if previous['status']=='published':
            try:
                current = lookup(row)
                validate_observed(row, current, snapshot['schemaVersion'])
                if snapshot['schemaVersion'] == 1:
                    fields = ['status', 'name', 'version', 'archiveSha256'] + (['integrity'] if row['kind'] == 'npm' else [])
                    fields += [field for field in ['archiveBytes', 'evidenceKind', 'tarballURL'] if field in previous]
                    same = all(current.get(field) == previous.get(field) for field in fields)
                    sections = ['dependencies', 'optionalDependencies', 'peerDependencies']
                    if 'peerDependenciesMeta' in previous.get('metadata', {}):
                        sections.append('peerDependenciesMeta')
                    same = same and all(current.get('metadata', {}).get(section, {}) == previous.get('metadata', {}).get(section, {}) for section in sections)
                else:
                    same = current == previous
                check(same,'Retained published version changed, disappeared or differs')
            except Exception as error:
                if not isolate:
                    raise
                failures[key] = unknown(row, error)
    return failures


def npm_dependencies(source,snapshot):
    """Check internal runtime/optional/peer declarations against the complete family.

    External dependencies remain subject to frozen-lock and candidate install checks.
    Published packages are checked using their registry declarations, not new source.
    """
    family=validate(source,'npm',snapshot);by_name={row['name']:row for row in family.values()};checks=[]
    for key,row in family.items():
        observed=snapshot['packages'][key]
        manifest=observed['metadata'] if observed['status']=='published' else json.loads(source['files'][row['manifestPath']])
        for section in ['dependencies','optionalDependencies','peerDependencies']:
            for name,value in manifest.get(section,{}).items():
                if name not in by_name:continue
                check(isinstance(value,str),'Invalid internal dependency range')
                if value.startswith('workspace:'):
                    value=value[len('workspace:'):]
                    if value in ['*','^','~']:value=by_name[name]['version']
                checks.append([by_name[name]['version'],value,row['name'],name])
    for version,requirement,package,dependency in checks:
        check(stable_satisfies(version,requirement),'Internal publication dependency incompatible: '+package+' -> '+dependency)


def stable_satisfies(version,requirement):
    """Deliberately bounded stable family ranges; unsupported syntax fails closed.

    Family manifests currently use exact versions and workspace:* / workspace:^.
    Published normalized equivalents may use * / ^ / ~. No tag or prerelease can
    silently authorize a different family member.
    """
    check(re.fullmatch(r'\d+\.\d+\.\d+',version),'Stable family dependency required')
    if requirement=='*':return True
    match=re.fullmatch(r'([~^=]?)(\d+)\.(\d+)\.(\d+)',requirement)
    check(match is not None,'Unsupported internal publication range: '+requirement)
    prefix=match[1];floor=tuple(map(int,match.groups()[1:]));actual=tuple(map(int,version.split('.')))
    if prefix in ['', '=']:return actual==floor
    if prefix=='~':ceiling=(floor[0],floor[1]+1,0)
    elif floor[0]:ceiling=(floor[0]+1,0,0)
    elif floor[1]:ceiling=(0,floor[1]+1,0)
    else:ceiling=(0,0,floor[2]+1)
    return floor<=actual<ceiling



def legacy_plan(source,family,snapshot):
    declared=validate(source,'npm',snapshot);npm_dependencies(source,snapshot)
    # Reuse the common complete-family and graph validation and dependency walk.
    result=chain.plan_chain('release',{'sha':source['sha'],'tree':source['tree']},source['files'],family,[],source_modes=source['modes'],publication_scope='npm')
    selected=sorted(key for key,row in snapshot['packages'].items() if row['status']=='missing')
    result.update(selectionPolicy='registry-exact-version-v1',registrySnapshot=snapshot,baselineSHA=None,
                  build=selected,publish=selected,versionRequired=[],unchanged=sorted(set(declared)-set(selected)),
                  publicationBlocked=False,publicationBlockReason=None,registryCollisionChecks=selected,
                  reasons=['exact-registry-version-difference'],fullBuild=len(selected)==len(declared),
                  impactTrace=[],directNodes=[],affectedNodes=[],testNodes=[],sameVersionRebuildPolicy='build-dependencies-without-republishing')
    result['baselineBinding']=snapshot.get('stateBaseline')
    result['registryRetained']={key:{**declared[key],**row} for key,row in snapshot['packages'].items() if row['status']=='published'}
    nodes,issues=chain.repository_graph(source['files']);check(not issues,'Publication graph unresolved')
    chain.build_closure(result,declared,nodes)
    result['planHash']=chain.digest({key:value for key,value in result.items() if key!='planHash'})
    return result


def native_cell(key, row):
    return {'owner': row['owner'], 'platform': row['platform'], 'package': key}


def presence_preparation(result, source, declared, snapshot):
    """Source JS tools may prepare missing products, but native cells are exact.

    Public platform packages are not a compiler dependency closure. A missing
    platform is built from its source owner. The host GNU tool is either another
    missing cell or separately verified registry-origin input, never both.
    """
    nodes, issues = chain.repository_graph(source['files'])
    check(not issues, 'Publication graph unresolved')
    chain.build_closure(result, declared, nodes)
    selected = set(result['build'])
    cells = [native_cell(key, declared[key]) for key in sorted(selected) if declared[key].get('owner')]
    js = {key for key in selected if not declared[key].get('owner')
          and not declared[key]['manifestPath'].startswith('rust-plugins/')
          and declared[key]['name'] != 'create-farm'}
    build_nodes = set(result['buildNodes'])
    pending = ['npm:packages/core'] if js else []
    while pending:
        key = pending.pop()
        if key in build_nodes:
            continue
        check(key in nodes, 'Release preparation node missing')
        build_nodes.add(key)
        pending.extend(nodes[key]['preparationDeps'])
    preparation, registry_inputs, blocked = [], {}, {}
    # Core's config loader executes replace-dirname, even when farm.config.ts
    # does not import it. Visualizer's client recipe additionally executes worker.
    # Runtime imports in docs are not executed by their tsc-only build recipes.
    owners = ['@farmfe/core', '@farmfe/plugin-replace-dirname'] if js else []
    if 'npm:js-plugins/visualizer' in build_nodes:
        owners.append('@farmfe/plugin-worker')
    for owner in owners:
        host = next((key for key, row in declared.items()
                     if row.get('owner') == owner and row.get('platform') == 'linux-x64-gnu'), None)
        if host is None:
            # Small graph fixtures need not contain the real native family.
            continue
        observed = snapshot['packages'][host]
        if observed['status'] == 'missing':
            preparation.append(native_cell(host, declared[host]))
        elif observed['status'] == 'published':
            registry_inputs[host] = {**declared[host], **observed, 'origin': 'registry-present'}
        else:
            affected = js if owner != '@farmfe/plugin-worker' else {
                key for key in js if declared[key]['manifestPath'] == 'js-plugins/visualizer/package.json'}
            for key in sorted(affected):
                blocked.setdefault(key, {'reason': 'registry-host-tool-unknown', 'dependencies': []})['dependencies'].append(host)
    # Do not expand an owner's optional platform inventory into native builds.
    payload_nodes = {'npm:' + str(Path(row['manifestPath']).parent).replace('\\', '/'): key
                     for key, row in declared.items() if row.get('owner')}
    source_payloads = {row['package'] for row in cells + preparation}
    build_nodes -= {node for node, key in payload_nodes.items() if key not in source_payloads}
    result.update(nativeBuildCells=cells, preparationNativeCells=preparation,
                  registryBuildInputs=registry_inputs, buildBlocked=blocked,
                  preparationNativeOwners=sorted({row['owner'] for row in cells + preparation}),
                  buildNodes=sorted(build_nodes),
                  buildEdges={key: sorted(nodes[key]['preparationDeps'] & build_nodes) for key in sorted(build_nodes)})
    result['buildDependencyClosure'] = sorted(key for key in result['buildDependencyClosure']
        if not declared[key].get('owner') or key in source_payloads)


def plan(source, family, snapshot):
    if snapshot.get('schemaVersion') == 1:
        return legacy_plan(source, family, snapshot)
    declared = validate(source, 'npm', snapshot)
    # Graph/family validation is source-global. Registry failures are package-local
    # and cannot stop unrelated missing versions from entering the build plan.
    result = chain.plan_chain('release', {'sha': source['sha'], 'tree': source['tree']}, source['files'], family, [],
                              source_modes=source['modes'], publication_scope='npm')
    selected = sorted(key for key, row in snapshot['packages'].items() if row['status'] == 'missing')
    retained = {key: {**declared[key], **row} for key, row in sorted(snapshot['packages'].items()) if row['status'] == 'published'}
    uncertain = {key: {**declared[key], **row} for key, row in sorted(snapshot['packages'].items()) if row['status'] == 'unknown'}
    result.update(selectionPolicy=POLICY, registrySnapshot=snapshot, baselineSHA=None, baselineBinding=None,
                  build=selected, publish=selected, versionRequired=[], unchanged=sorted(retained),
                  publicationBlocked=False, publicationBlockReason=None, registryCollisionChecks=selected,
                  reasons=['exact-registry-version-presence'], fullBuild=len(selected) == len(declared),
                  impactTrace=[], directNodes=[], affectedNodes=[], testNodes=[],
                  sameVersionRebuildPolicy='source-preparation-only-no-native-rebuild-or-republication',
                  registryRetained=retained, registryUnknown=uncertain)
    presence_preparation(result, source, declared, snapshot)
    result['planHash'] = chain.digest({key: value for key, value in result.items() if key != 'planHash'})
    return result
