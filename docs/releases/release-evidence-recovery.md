# Fixed-main evidence recovery

This workflow recovers a release blocked solely by the bounded wait for main CI.
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
its exact current-attempt artifact, successful upload step, timestamps, ZIP and
digest, source tree and intent not to publish. Missing or expired proof blocks
recovery. Large native inventories do not affect the exact-name lookup.

## Only unfrozen transactions

Every earlier release attempt must prove that the publisher was skipped and no
candidate freeze executed. Any candidate or seal selects the existing frozen
recovery path instead. Other failed or cancelled jobs also block automatic
recovery. The controller only requests `rerun-failed-jobs` for the verified
original run. Successful native jobs are carried, not uploaded again.

The source plan's original attempt and the candidate producer's actual attempt
are separate outputs. A candidate identity job preserves the frozen producer
attempt for all consumers, publisher, public proof and finalization step. For example,
plan 1 plus native 1 can feed producer 2; consumer attempt 3 still reads candidate
2. A full manual rerun uses the existing original-archive verification instead.

## Races and boundaries

The controller rechecks its complete decision immediately before POST. Per-SHA
concurrency serializes events; a duplicate observing a queued or running attempt
is a no-op. Both release completion and CI completion are wake-ups, covering the
race where CI succeeds just after the 120-minute wait expires. Recovery is bounded
to 20 attempts, and a new verification failure without a new pending receipt does
not trigger a retry loop. A concurrent external rerun after the last read may
make the POST fail; that error is not treated as acceptance or retried blindly.

Main moving forward does not change the run source. Existing fixed-source, registry
conflict, durable lock and original-byte recovery checks remain mandatory.
Frozen partial publications never rebuild replacements through this controller.

The new controller declares `actions: write` solely for the original-run retry;
its token has read-only contents and no npm or Cargo token. Local fixtures do not
prove hosted permissions, upload timing, carried outputs or actual publication.
The implementation and tests may run locally without invoking the controller CLI.
