"""Prove a failed release attempt never froze candidates or ran a publisher."""
def require(value, message):
    if not value:
        raise ValueError(message)


def verify_unfrozen_retry(get, *, run_id, source_sha, current_attempt, workflow, producer, freeze_step, publisher, events):
    # This is deliberately bounded and fail-closed. It authorizes preparation,
    # never upload, and cannot recover a lost successfully frozen transaction.
    require(type(current_attempt) is int and 1 < current_attempt <= 20, 'Unfrozen retry attempt bound exceeded')
    repository = get('')
    require(type(repository.get('id')) is int and repository['id'] > 0, 'Prior release repository identity is unproven')
    receipts = []
    for attempt in range(1, current_attempt):
        route = '/actions/runs/' + str(run_id) + '/attempts/' + str(attempt)
        run = get(route)
        require(run['id'] == run_id and run['run_attempt'] == attempt and run['path'] == workflow
                and run['head_sha'] == source_sha and run['status'] == 'completed'
                and run['event'] in events and run['repository']['id'] == repository['id'] == run['head_repository']['id'],
                'Prior release attempt identity or completion is unproven')
        response = get(route + '/jobs?per_page=100')
        jobs = response['jobs']
        require(type(response.get('total_count')) is int and response['total_count'] == len(jobs) and len(jobs) <= 100,
                'Prior release job listing is incomplete')
        def exact(name):
            matches = [job for job in jobs if job['name'] == name]
            require(len(matches) == 1, 'Exact prior release job is missing or ambiguous: ' + name)
            job = matches[0]
            require(job['run_id'] == run_id and job['status'] == 'completed', 'Prior release job is not terminal')
            require(job.get('run_attempt', attempt) == attempt and job.get('head_sha', source_sha) == source_sha,
                    'Prior release job source or attempt differs')
            return job
        write = exact(publisher)
        require(write['conclusion'] == 'skipped' and write.get('steps') == [],
                'Prior publisher may have executed; original frozen archives are required')
        build = exact(producer)
        require(build['conclusion'] in ['failure', 'cancelled', 'skipped'], 'A successful producer cannot be rebuilt without its archives')
        if build['conclusion'] == 'skipped':
            require(build.get('steps') == [], 'Skipped producer has unexpected executed steps')
        else:
            freeze = [step for step in build['steps'] if step['name'] == freeze_step]
            require(len(freeze) == 1 and freeze[0]['status'] == 'completed' and freeze[0]['conclusion'] == 'skipped',
                    'Candidate freeze may have executed; refuse to rebuild missing originals')
        receipts.append({'runId': run_id, 'attempt': attempt, 'publisher': 'skipped', 'freeze': 'not-executed'})
    return receipts
