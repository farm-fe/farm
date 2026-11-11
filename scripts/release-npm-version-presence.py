"""Stateless publication of missing versions and registry-origin tag confirmation."""
# cspell:words argparse importlib readback npmjs ENEEDAUTH retryable
from contextlib import nullcontext
from datetime import datetime, timezone
import hashlib
import math
import re
import time


def is_presence(manifest):
    return manifest.get('selectionPolicy') == 'registry-exact-version-v2'


def report(manifest, base):
    check = base.check
    check(is_presence(manifest) and manifest.get('baseline') is None and manifest.get('publicationScope') == 'npm',
          'Stateless npm version policy required')
    for key, length in [('sourceSHA', 40), ('sourceTree', 40), ('planHash', 64)]:
        check(re.fullmatch('[a-f0-9]{' + str(length) + '}', manifest.get(key, '')), 'Frozen source identity missing')
    family = manifest.get('family'); snapshot = manifest.get('registrySnapshot', {})
    check(isinstance(family, list) and 0 < len(family) <= 1024, 'Complete npm family required')
    names = {row['kind'] + ':' + row['name']: row for row in family}
    check(len(names) == len(family) and all(row['kind'] == 'npm' for row in family), 'Duplicate or non-npm family')
    check(snapshot.get('schemaVersion') == 2 and snapshot.get('stateBaseline') is None
          and snapshot.get('sourceSHA') == manifest['sourceSHA'] and snapshot.get('sourceTree') == manifest['sourceTree']
          and set(snapshot.get('packages', {})) == set(names), 'Complete source-bound registry observation required')
    for key, row in names.items(): base.selector.validate_observed(row, snapshot['packages'][key], 2)
    archives = {row['kind'] + ':' + row['name']: row for row in manifest.get('packages', [])}
    missing = {key for key, row in snapshot['packages'].items() if row['status'] == 'missing'}
    failures = manifest.get('buildFailures', {})
    check(isinstance(failures, dict) and len(archives) == len(manifest['packages'])
          and not set(archives).intersection(failures) and set(archives) | set(failures) == missing
          and set(manifest['publish']) == set(manifest['rebuild']) == missing, 'Missing version build partition differs')
    for key, row in archives.items():
        check(row['version'] == names[key]['version'] and row['kind'] == 'npm'
              and re.fullmatch('[a-f0-9]{64}', row.get('archiveSha256', ''))
              and type(row.get('archiveBytes')) is int and 0 < row['archiveBytes'] <= 100 * 1024 ** 2,
              'New archive identity invalid')
    outcomes = []
    for key, row in names.items():
        observed = snapshot['packages'][key]
        outcome = {'name': row['name'], 'version': row['version'], 'status': 'pending',
                   'uploadAttempted': False, 'outcomeUncertain': False, 'registryBytesVerified': False,
                   'originKind': 'registry' if observed['status'] == 'published' else 'source', 'errors': []}
        if observed['status'] == 'unknown':
            outcome.update(status='failed', originKind='unknown', errors=[{'stage': 'registry-selection', 'code': observed['reason']}])
            if 'priorSubmission' in observed: outcome['priorSubmission'] = observed['priorSubmission']
        elif key in failures:
            outcome.update(status='failed', errors=[{'stage': 'build', 'code': failures[key]}])
        elif key in archives:
            outcome['candidateArchiveSha256'] = archives[key]['archiveSha256']
        outcomes.append(outcome)
    return {'schemaVersion': 2, 'kind': 'npm-package-publication-results', 'stage': 'publisher',
            'selectionPolicy': manifest['selectionPolicy'], 'manifestHash': base.entry.chain.digest(manifest),
            'sourceSHA': manifest['sourceSHA'], 'sourceTree': manifest['sourceTree'],
            'producerRunId': manifest['producerRunId'], 'producerAttempt': manifest['producerAttempt'],
            'complete': False, 'success': False, 'baselineAdvanced': False, 'releaseCreatedByThisStage': False,
            'packages': outcomes}


def command_outcome(error=None):
    return {'operation': 'publish', 'returncode': 0 if error is None else getattr(error, 'returncode', None),
            'timedOut': False if error is None else getattr(error, 'timed_out', False),
            'npmCodes': [] if error is None else list(getattr(error, 'npm_codes', ())),
            'accepted': error is None}


def publish(manifest, archive_reader, registry, result, save, base, *, remaining=lambda: 4200,
            clock=time.monotonic, sleep=time.sleep, visibility_seconds=1200):
    check = base.check
    check(type(visibility_seconds) is int and 0 < visibility_seconds <= 1800, 'Invalid registry visibility budget')
    family = {row['name']: dict(row) for row in manifest['family']}
    source = {row['name']: row for row in manifest['packages']}
    original = {key[4:]: row for key, row in manifest['registrySnapshot']['packages'].items()}
    outcomes = {row['name']: row for row in result['packages']}
    metadata, observed, dependencies, peers, closures = {}, {}, {}, {}, {}
    available, pending, waiting = set(), [], {}
    used, limits = {}, {name: 100 * 1024 ** 2 for name in family}
    reader = getattr(registry, 'reader', None)

    def stamp(): return datetime.now(timezone.utc).isoformat()

    def failure(name, stage, error=None, blocked=None):
        row = outcomes[name]; row['status'] = 'failed'
        row['errors'].append(base.diagnostic(stage, error) if error is not None else
                             {'stage': stage, **({'dependencies': sorted(blocked)} if blocked is not None else {})})
        save(result)

    def set_limits():
        reads = {name: 6 for name in family}
        for name in metadata:
            for dependency in dependencies.get(name, set()): reads[dependency] += 1
            for dependency in closures.get(name, set()): reads[dependency] += 1
        for name in family:
            size = observed.get(name, source.get(name, {})).get('archiveBytes', 0)
            limits[name] = 100 * 1024 ** 2 + size * reads[name]
        if reader is not None:
            reader.total_byte_limit = min(1024 ** 4, max(3 * 1024 ** 3, sum(limits.values()) + len(family) * 1024 ** 2))

    def fetch(name):
        remaining()
        if reader is None:
            value = registry.observe(family[name], verify_archive=outcomes[name]['uploadAttempted'])
        else:
            before, global_limit = reader.total_bytes, reader.total_byte_limit
            allowance = limits[name] - used.get(name, 0)
            check(allowance > 0, 'Registry package byte budget exceeded')
            reader.total_byte_limit = min(global_limit, before + allowance)
            try:
                value = registry.observe(family[name], verify_archive=outcomes[name]['uploadAttempted'])
            except ValueError:
                if before + allowance <= global_limit and reader.total_bytes > before + allowance:
                    raise ValueError('Registry package byte budget exceeded') from None
                raise
            finally:
                used[name] = used.get(name, 0) + reader.total_bytes - before
                outcomes[name]['registryReadBytes'] = used[name]
                reader.total_byte_limit = global_limit
        base.selector.validate_observed(family[name], value, 2)
        check(value['status'] in ['missing', 'published'], 'Registry read is not a publication decision')
        return value

    def remember(name, value, *, race=False):
        check(value['status'] == 'published', 'Public package version missing')
        outcome = outcomes[name]
        if outcome['uploadAttempted']:
            check(value['archiveSha256'] == source[name]['archiveSha256']
                  and value['archiveBytes'] == source[name]['archiveBytes'], 'Immutable registry version has different bytes')
        elif original[name]['status'] == 'published':
            check(all(value[field] == original[name][field] for field in
                      ['integrity', 'metadata', 'tarballURL']), 'Retained registry observation changed')
        if race:
            check(not outcome['uploadAttempted'], 'Attempted upload cannot become a retained race')
            outcome['originKind'] = 'registry-race'
        observed[name] = value; metadata[name] = value['metadata']; available.add(name)
        fields = ['integrity', 'metadata', 'tarballURL', 'evidenceKind']
        if outcome['uploadAttempted']:
            check(value.get('evidenceKind') == 'registry-archive', 'Uploaded bytes require archive readback')
            fields += ['archiveSha256', 'archiveBytes']
        else:
            check(value.get('evidenceKind') == 'registry-metadata', 'Retained version must not claim downloaded bytes')
        outcome.update({field: value[field] for field in fields})
        outcome.update(registryBytesVerified=outcome['uploadAttempted'], registryMetadataVerified=True, outcomeUncertain=False, registryVerifiedAt=stamp())
        if not outcome['errors']: outcome['status'] = 'registry-verified'
        outcome['outcome'] = 'published' if outcome['uploadAttempted'] else 'retained'
        save(result)

    def same_observation(name, value):
        fields = ['name', 'version', 'integrity', 'metadata', 'tarballURL', 'evidenceKind']
        if outcomes[name]['uploadAttempted']: fields += ['archiveSha256', 'archiveBytes']
        return value['status'] == 'published' and all(value.get(field) == observed[name].get(field) for field in fields)

    def graph():
        dependencies.clear(); peers.clear(); closures.clear()
        for name in list(metadata):
            try:
                dependencies.update(base.required_dependencies(list(family.values()), {name: metadata[name]}))
                peers.update(base.required_dependencies(list(family.values()), {name: metadata[name]}, peers_only=True))
            except Exception as error:
                metadata.pop(name); available.discard(name); failure(name, 'dependency-validation', error)
        for name in metadata:
            seen = {name}; todo = list(dependencies.get(name, set()) | peers.get(name, set()))
            while todo:
                dependency = todo.pop()
                if dependency in seen: continue
                seen.add(dependency)
                todo.extend(dependencies.get(dependency, set()) | peers.get(dependency, set()))
            closures[name] = seen - {name}
        set_limits()

    def enqueue(name, phase):
        waiting[name] = {'phase': phase, 'end': clock() + visibility_seconds, 'next': clock(), 'delay': 2}
        outcomes[name][phase + 'Checks'] = 0; save(result)

    def observe(name):
        wait = waiting[name]; phase = wait['phase']; outcome = outcomes[name]
        if clock() < wait['next']: return False
        remaining()
        if clock() >= wait['end']:
            waiting.pop(name); failure(name, phase + '-timeout'); return True
        outcome[phase + 'Checks'] += 1; outcome[phase + 'CheckedAt'] = stamp()
        try:
            scope = reader.budget(max(1, math.ceil(wait['end'] - clock()))) if reader else nullcontext()
            with scope:
                tag_ready = phase == 'registry-readback' or registry.latest_visible(family[name])
                value = fetch(name) if tag_ready else None
            if value and value['status'] == 'published':
                remember(name, value)
                waiting.pop(name)
                if phase == 'latest-readback':
                    outcome.update(latestVerified=True, latestOutcomeUncertain=False, latestVerifiedAt=stamp())
                    if not outcome['errors']: outcome['status'] = 'success'
                graph(); save(result); return True
            outcome[phase + 'AbsentReads'] = outcome.get(phase + 'AbsentReads', 0) + 1
        except Exception as error:
            if not base.retryable_read(error):
                waiting.pop(name); failure(name, phase, error); return True
            outcome[phase + 'TransientErrors'] = outcome.get(phase + 'TransientErrors', 0) + 1
            outcome[phase + 'LastHttpStatus'] = getattr(error, 'code', getattr(error, 'status', None))
        wait['next'] = min(wait['end'], clock() + wait['delay']); wait['delay'] = min(30, wait['delay'] * 2)
        save(result); return False

    def promote_ready():
        progress = False
        future = set(pending) | {name for name, wait in waiting.items() if wait['phase'] == 'registry-readback'}
        for name, outcome in outcomes.items():
            if outcome['status'] != 'registry-verified' or name in waiting: continue
            missing = closures.get(name, set()) - available
            invalid = closures.get(name, set()) - set(metadata) - future
            if invalid or missing - future:
                failure(name, 'required-dependency-unverified', blocked=invalid | (missing - future)); progress = True; continue
            if missing: continue
            try:
                for dependency in sorted(closures.get(name, set()) | {name}):
                    value = fetch(dependency)
                    check(same_observation(dependency, value), 'Verified registry dependency changed')
            except Exception as error:
                failure(name, 'required-dependency-unverified', error); progress = True; continue
            rejected = False
            try:
                remaining(); outcome.update(status='awaiting-latest', latestOutcomeUncertain=True, latestConfirmationStartedAt=stamp())
                save(result); registry.promote_latest(family[name])
            except Exception as error:
                rejected = base.definite_rejection(error); failure(name, 'latest-confirmation', error)
            enqueue(name, 'latest-readback'); observe(name)
            if rejected and name in waiting: waiting.pop(name)
            progress = True
        return progress

    # Classify one new version only after progressing already-ready packages.
    # Never wait for unrelated initial metadata before submitting an independent
    # ready archive or confirming its latest tag.
    pending.extend(name for name in family if outcomes[name]['status'] != 'failed')
    initialized = set()

    def initialize(name):
        try:
            value = fetch(name)
            if value['status'] == 'published':
                pending.remove(name)
                remember(name, value, race=original[name]['status'] == 'missing')
            elif original[name]['status'] == 'published':
                pending.remove(name); failure(name, 'retained-version-not-visible')
            else:
                row = source[name]; body = archive_reader(row)
                check(len(body) == row['archiveBytes'], 'Frozen archive size differs')
                metadata[name] = base.publisher.npm_tarball.validate(row, body)
        except Exception as error:
            if name in pending: pending.remove(name)
            failure(name, 'registry-or-candidate-preflight', error)
        graph(); promote_ready()
        initialized.add(name)

    while pending or waiting or any(row['status'] == 'registry-verified' for row in outcomes.values()):
        progress = False
        for name in list(waiting): progress = observe(name) or progress
        progress = promote_ready() or progress
        for name in pending[:]:
            if name not in initialized: continue
            for other in list(waiting): progress = observe(other) or progress
            progress = promote_ready() or progress
            outcome = outcomes[name]
            if outcome['status'] == 'failed': pending.remove(name); progress = True; continue
            try:
                value = fetch(name)
                if value['status'] == 'published':
                    pending.remove(name); remember(name, value, race=True); graph(); progress = True; continue
            except Exception as error:
                pending.remove(name); failure(name, 'registry-before-upload', error); progress = True; continue
            required = dependencies.get(name, set())
            if (required - available).intersection(set(pending) | set(waiting)): continue
            pending.remove(name); progress = True
            if required - available:
                failure(name, 'required-dependency-unverified', blocked=required - available); continue
            try:
                for dependency in sorted(required):
                    value = fetch(dependency)
                    check(same_observation(dependency, value), 'Verified registry dependency changed')
                row = source[name]; body = archive_reader(row)
                check(len(body) == row['archiveBytes'] and hashlib.sha256(body).hexdigest() == row['archiveSha256'], 'Frozen upload bytes changed')
                base.publisher.npm_tarball.validate(row, body)
            except Exception as error:
                failure(name, 'upload-preparation', error); continue
            rejected = False
            try:
                remaining(); outcome.update(uploadAttempted=True, outcomeUncertain=True, status='upload-started', uploadStartedAt=stamp())
                save(result); registry.publish(row, body)
                outcome.update(publishCommandSucceeded=True, publishCommandCompletedAt=stamp(), status='awaiting-registry',
                               publishOutcome={**command_outcome(), 'definiteRejection': False})
            except Exception as error:
                rejected = base.definite_rejection(error)
                if rejected: outcome['outcomeUncertain'] = False
                outcome['publishOutcome'] = {**command_outcome(error), 'definiteRejection': rejected}
                failure(name, 'publish', error)
            enqueue(name, 'registry-readback'); observe(name)
            if rejected and name in waiting: waiting.pop(name)
            progress = promote_ready() or progress
        uninitialized = next((name for name in pending if name not in initialized), None)
        if uninitialized is not None:
            initialize(uninitialized); progress = True
        if not progress and not waiting and pending:
            for name in pending: failure(name, 'required-dependency-cycle', blocked=dependencies.get(name, set()) - available)
            pending.clear()
        elif not progress and waiting:
            sleep(min(max(0, min(item['next'] for item in waiting.values()) - clock()), remaining()))
    result.update(complete=True, success=all(row['status'] == 'success' for row in outcomes.values()))
    result['counts'] = {'total': len(outcomes), 'success': sum(row['status'] == 'success' for row in outcomes.values()),
                        'failed': sum(row['status'] == 'failed' for row in outcomes.values()),
                        'published': sum(row['status'] == 'success' and row['uploadAttempted'] for row in outcomes.values()),
                        'retained': sum(row['status'] == 'success' and not row['uploadAttempted'] for row in outcomes.values())}
    save(result); return result
