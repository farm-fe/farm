<!-- cspell:words finalizer unstarted -->
# Independent incremental CI and publication

CI and release share impact rules, never build bytes. The CI namespace is
`farm-ci-v1`, with profile `ci`. The release namespace is `farm-release-v1`, with
profile `release`. Unit `ci-test` and Docs CI-profile quality checks remain
inside CI. There is no third PR release-reference chain. Candidate, SDK and
consumer validation are internal stages of the one fixed-main Release workflow.
Neither chain imports artifacts from the other.

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
from npm owner-directory reverse dependency closure and ordinary committed engineering
Markdown under `docs/experiences` (Git blob mode `100644` on both present sides). Unknown inputs run all current tests.
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
keeps its original archives. A successful plan with both `build=[]` and `publish=[]` is an explicit workflow
no-op, not an empty publisher transaction. Recipe-only rebuilds have an independent
`build_required` output: they execute release producers and compare every rebuilt
archive with its independently verified original bytes. Their verification report
uses a separate artifact namespace, creates no publisher manifest and never advances
the published baseline. Terminal verification rechecks the selection digest, fixed
source/tree and trusted planner job hash after package build scripts have run.
Private build prerequisites execute through the planner's resolved `buildEdges`,
including file/link/workspace aliases; public prerequisites not selected for rebuild
remain materialized original release archives. Same-version rebuilt archive bytes
must equal the independently verified original, including mixed publication
batches. Removed packages require an explicit retirement contract and currently
block publication.


## Two workflow chains and rerun evidence

Ordinary PRs trigger only CI (`ci.yaml`, Unit and ordinary quality checks).
CI builds with its own CI recipes, runs selected TS/Examples/E2E suites and Docs
quality checks, and records original execution evidence. It does not call a
release candidate seal or dispatch the main Release workflow from a PR branch.
The former PR release-validation workflow and namespace have been retired.

`incremental-release.yaml` runs only against a fixed main snapshot. A main push
or explicit main dispatch follows the same release plan, release-profile build,
seal, SDK and candidate-consumer stages. `publish=false` is a dry-run of that
same chain; registry writes still require explicit `publish=true`, exact source
and profile proof, the publication fence, conflict checks and public acceptance.
Version preparation may validate its PR through CI; it cannot launch a release
reference on that PR. Release ABI/profile checks remain inside Release.

The shared impact rules do not permit CI and Release artifacts to cross chains.
Same-source reruns retain each successful producer's actual job ID, original
attempt, upload step, immutable artifact name and digest. A present but unstarted/queued,
skipped, cancelled or failed newer job cannot fall back to an older success.
Carried jobs require completed/success, the same job ID and identical times.
CI/Unit impact plans, suite selections, suite receipts, performance reports,
summary/ledger and GNU materializations all resolve their original producers.
Restoring a materialization never rewrites its receipt as a new native build or
inherits earlier test acceptance; current consumers check the original bytes.

A new HEAD must obtain its own real CI result. These workflow/recipe changes do
not authorize importing the old HEAD's native products. After a successful
fixed-HEAD execution, rerun only the failed leaf performance job or the relevant
gate/summary: those subgraphs do not rerun upstream native producers. Do not use
a Type Check or primary native producer as an aggregation retry shortcut.
Old-run local diagnostics cannot stand in for new-HEAD GitHub checks or baselines.

### Directory selection contract

CI suite selection uses `scripts/ci-directory-impact-rules.mjs`: global paths
select all suites; other paths map to npm owner directories and propagate through
production, development, optional and peer dependencies, plus the declared
cross-directory validation edges. Both baseline and current manifests participate
so removals and renames retain old consumers. Local file/link/workspace aliases
resolve to directory identities. Unknown paths or unresolved dependencies select
all suites. Decisions include path/owner/dependency explanations.

This affects actual TS, Examples build and browser selection only after the
independent same-chain baseline check; it does not authorize acceptance from a
plan alone. Create-farm still validates all eight templates. Rust Unit, native
production, Docs quality and performance remain full. Shared compiler/runtime
inputs and unowned SSR helpers are explicit global inputs. General JS preparation
is still the existing bootstrap recipe; selective prerequisite builds are a
separate remaining step. The release graph already parses Cargo and npm manifests,
but unifying its directory policy with this suite rule inventory remains pending.
New cross-directory reads require a declared dependency edge or a global rule;
there is no AST analysis. PostCSS's standalone cache integration test is not part
of the ordinary TS suite and is not claimed as executed by this selection.
