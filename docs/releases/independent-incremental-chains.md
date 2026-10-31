# Independent incremental CI and publication

CI and release share impact rules, never build bytes. The CI namespace is
`farm-ci-v1`, with profile `ci`. The release namespace is `farm-release-v1`, with
profile `release`. Unit `ci-test` and Docs release-profile reference builds are
separate acceptance tasks; a release profile alone does not authorize their bytes
for publication. Neither chain imports artifacts from the other.

## First implementation batch

`scripts/incremental-chain-plan.py` provides a shared selection calculation and
an exact Git source CLI for the no-baseline fallback. Directory-based node
identities preserve private Examples with the same npm name. Actual ambiguous
dependency edges require a full build instead of merging package identities.
Old and new dependency edges are joined. Observed snapshot differences supplement
the caller's changed paths, including deletion and rename endpoints. Both bytes
and Git executable modes participate in that comparison and source fingerprint.
The full family is independently derived from public owner manifests, the native
support recipe and the four SDK roots' production path dependency closure. A
caller cannot shorten the family or baseline package set to hide missing work.
Unresolved local dependency protocols require a full build.
Workspace aliases are not silently resolved as same-name packages. The forward
walk crosses private/non-published nodes before projecting the publish family;
the executor also receives those private build nodes. A conservative full build
does not accidentally add the independently excluded copy package to publication
scope. Direct edits to that package still require a scope review.

The result separates build, version-required, publish and unchanged packages.
A recipe change can require a full rebuild without forcing every unchanged
package to a new version. Source or dependency changes at an already published
version block publication pending version preparation. Farm ABI changes require
all affected native owners and their payloads to advance; the independent
create-farm Node-API interface is outside the Farm plugin ABI cohort.
Those owners are also reverse-dependency roots, so unchanged consumers of a new
ABI cannot silently retain their old release. The forward build dependency
closure is separate from the changed-package set. A same-version recipe rebuild
must compare original registry archive bytes; different bytes require a new
version, never an overwrite.

A release baseline must identify a completed successful release, exact source
and tree, complete package records and independent registry checksums. A CI
baseline cannot substitute for it. The baseline anchor argument is a trusted
adapter input, like the existing native proof GitHub evidence input. Reading a
JSON file containing `verified: true` is not an implementation of that adapter.
There is deliberately no CLI option accepting a user-provided trusted anchor.

The contracts are connected to the blocking source-contract job and have local
lightweight model, mock, archive and isolated-Git coverage. The generic selection
object is not publication authority. The live CI entrypoints now support GNU
same-chain byte reuse for engineering-documentation leaves and selected suites
for the audited React/Babel source leaves. Unknown inputs run all current tests.
The independent release workflow consumes the source-bound rebuild/version/publish
plan, builds its own release-profile bytes and requires explicit main publication
authority. None of this documents a completed hosted release or benchmark run.

## Execution contract

1. Read successful same-chain baseline evidence with the existing authorized
   Actions identity. Bind source, tree, policy, producer run/attempt/job and
   original artifact ID/digest. Validate complete inputs, recipe, toolchain,
   target, host, profile, flags and ABI before same-chain byte reuse. Missing,
   legacy, expired or inaccessible evidence rebuilds within the current chain.
2. Connect CI selections to real platform producers and semantic suites. Every
   omitted execution must have an explicit not-required decision or a verified
   same-chain byte receipt. Acceptance-script changes rerun acceptance; prior
   test success is never inherited. Stable checks validate the entire planned
   inventory and actual result counts, not an empty successful matrix.
3. On trusted main, acquire the shared publication lock, fix the source snapshot
   and read the latest successful release baseline after acquiring that lock.
   The first release has no baseline and independently builds the complete
   release family. Subsequent releases build the affected release closure.
   Changesets and Cargo version preparation must match the calculated publish
   delta, including native owner/payload dependency updates.
4. Build and seal release bytes independently of PR CI. Keep licenses, all nine
   native targets, the Rust SDK, independent consumers and real three-platform
   business-2048 evidence. Upload Rust before dependent npm packages. Verify
   immutable registry bytes and latest-tag targets, then complete the Release
   and successful-release ledger. Record partial per-package progress for safe
   continuation; a partial upload never becomes a successful baseline.

Concurrent merged PRs cannot share a pre-lock release plan. Recompute against the
successful baseline while holding the publication lock, reject source/version
collisions and do not move an existing version or tag. A newer main commit is
another fixed snapshot, not permission to replace bytes in the current run.

The canceled separate full-build PR is not a prerequisite. Its read-only source
may inform release-profile proof handling, but no entire change set is imported
and its workspace is not modified.

## Durable published baseline and recovery

`incremental-release-baseline.py` independently checks the original release
producer/seal, per-platform candidate acceptance across attempts, public acceptance,
registry bytes and exact tag/Release. Candidate receipts preserve their original
producer and bind the current consumer attempt, OS, architecture and ABI.

The finalizer writes a bounded compressed repository receipt in the GitHub
Release body and reads it back before advancing the durable successful baseline.
It is a repository publication record, not a signature or Actions attestation.
The reader requires the repository Actions publisher, exact source/tag/main
history and complete family; it still re-reads original public registry bytes.
This complete checkpoint removes the dependency on a chain of 90-day artifacts
and the 32-transaction legacy walk. Missing legacy evidence cannot authorize
reuse. The durable Git state remains bounded; a state-size limit stops safely
rather than silently discarding transaction history.

Registry transport URLs do not enter the semantic plan hash. Original source,
versions, archive digests, producer identity and baseline transaction do. The
publisher re-reads and recomputes the plan while owning the publication fence.
A moved baseline rejects the frozen candidate; a partly uploaded transaction
keeps its original archives. A successful no-change plan is an explicit workflow
no-op, not an empty publisher transaction. Same-version rebuilt archive bytes
must equal the independently verified original, including mixed publication
batches. Removed packages require an explicit retirement contract and currently
block publication.
