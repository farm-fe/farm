# Fixed-main evidence recovery

This workflow recommends manual recovery for a release blocked solely by the bounded wait for main CI.
It does not authorize publication, dispatch a new source, or retry general build
failures. Registry writes remain inside the original release workflow and lock.

## Wake-up and trust

Completion of E2E Tests, Unit Tests, or the independent release workflow wakes a
main-only controller. It checks out the controller workflow commit, never event
source code. It validates actual API workflow IDs and paths, repository IDs,
original push event, main branch, exact SHA, run ID and attempt. A display name
alone is not proof. The latest exact-SHA CI and Unit executions must succeed;
old success cannot conceal a newer failure, cancellation or running attempt.

A timeout produces a pending receipt, not functional acceptance. Recovery checks
its exact current-attempt artifact, signed upload identity, original attempt, successful upload step, ZIP and
digest, source tree and intent not to publish. Missing or expired proof blocks
recovery. Large native inventories do not affect the exact-name lookup.

## Only unfrozen transactions

Every earlier release attempt must prove that the publisher was skipped and no
candidate freeze executed. Any candidate or seal selects the existing frozen
recovery path instead. Other failed or cancelled jobs also block the recommendation. The controller
returns `manual-recovery-required` with `attempt-conditional-rerun-unavailable`,
`requested: false`, and the verified run/source/attempt. A maintainer must inspect
the current attempt and initiate recovery; the controller never sends a POST.

The source plan's original attempt and the candidate producer's actual attempt
are separate outputs. A candidate identity job preserves the frozen producer
attempt for all consumers, publisher, public proof and finalization step. For example,
plan 1 plus native 1 can feed producer 2; consumer attempt 3 still reads candidate
2. A full manual rerun uses the existing original-archive verification instead.

## Races and boundaries

The controller rechecks its complete decision before returning a recommendation.
This does not eliminate the last-read race: GitHub's run-ID rerun endpoint has no
attempt compare-and-swap. A newer manual attempt may freeze or partially publish
before a POST would run. Therefore there is no automatic mutation, even if the
endpoint would accept it with HTTP 201. Per-SHA concurrency does not serialize
manual reruns. Recommendations are observations, not execution authorization.
Both release and CI completion remain wake-ups. Validation is bounded to 20
attempts; a new failure without a pending receipt cannot authorize recovery.

Main moving forward does not change the run source. Existing fixed-source, registry
conflict, durable lock and original-byte recovery checks remain mandatory.
Frozen partial publications never rebuild replacements through this controller.

The existing workflow token declaration is unchanged in this repair, but the
controller has no write method or POST path. It has no npm or Cargo token. Local fixtures do not
prove hosted permissions, upload timing, carried outputs or actual publication.
The implementation and tests may run locally without invoking the controller CLI.
