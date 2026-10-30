# Incremental CI rollout

The first implementation is shadow-only. No native build, test, Docs acceptance,
Examples case, performance sample or release gate is omitted. It adds a plan and
an informational summary to the existing workflow. This document does not grant
publication approval.

Only the newly added shadow planning/reporting jobs tolerate errors. Their
artifacts preserve unavailable/error states. Existing functional jobs and all
publication requirements retain their previous failure behavior. A failed-jobs
rerun without a fresh plan or all current-attempt receipts is reported incomplete,
not silently bound to an older attempt.

## Committed input planning

`scripts/ci-impact-plan.py` reads exact Git blobs, not the dirty working tree.
It records base, merge base, head, actual tested commit and trees, run and attempt,
policy and plan digest. Merge base, current base, head and actual tested merge
graphs contribute edges and all deepest file owners, including dual Cargo/npm
directories. Deleted and renamed paths are included. Cargo normal/build/test
dependencies, target tables, repository path dependencies and JavaScript workspace
dependencies contribute conservative reverse closures. Optional features and
target conditions are not pruned in this first version.

Local test-source and conservative product fingerprints are separate fields.
Test-looking paths remain product inputs unless a separately reviewed policy
proves they are not read by a build or generator; the live planner has no such
exclusions. Acceptance fingerprints include dependency, runner and configuration
inputs and explicitly mark dynamic completeness unproven. Rust tests embedded in
`src` remain product inputs; no text parser strips conditional Rust code. Unknown
files, changed build policies, malformed manifests, missing objects or ambiguous
merge bases require full execution. Dynamic imports and generated dependencies
not yet represented in the graph must not authorize omission.

The plan contains proposed platform-local dependencies and Docs semantic suite
selection. They are reviewable data, not replacements for current workflow needs.
The live producer matrices and all current release acceptance remain unchanged.

## Native bytes are not acceptance

`scripts/ci-native-proof.py` implements bounded manifest creation, archive
verification and a build-on-error resolver. Context includes distinct `ci`,
`ci-test` and `release` profiles, exact target/ABI, measured host/toolchain/linker,
image identity, flags, features, lock, source closure and build policy hashes.
Archives have an exact product set, immutable GitHub artifact ID and digest,
bounded ZIP members and expanded bytes, no path traversal or symbolic links.

The consumer binding preserves `originBuildSHA`, original tree, producer run,
attempt, job and workflow. It separately binds consumer SHA, tested commit/tree,
merge tree and plan hash. A successful producer in a failed aggregate run may
supply byte evidence; a failed, cancelled or incomplete producer cannot. No test
or performance result is inherited from reused bytes. Legacy artifacts without
the complete proof always require building.

GitHub run `head_sha` is not assumed to be the actual checkout SHA. An authorized
read-only adapter must independently obtain run/attempt/job/artifact identities,
the actual producer Git commit and tree, the input closure and reviewed workflow
and producer code. PR-body claims and a receipt are not a signature or a trusted
substitute for those API and Git facts. Fork producer output is not eligible.

**Live reuse remains disabled.** Producers now capture actual tool versions,
whitelisted flag digests, exact binary hashes and committed source ledgers.
Capture runs inside the existing container when applicable. These small shadow
records are uploaded without duplicating native archives. An implicit or
unmeasured linker and unresolved final compiler features keep context incomplete;
the record is deliberately not a reusable producer proof.

The authenticated read-only adapter is connected to the assembly job's existing
`actions: read` access for current-run identity auditing. It distinguishes run
head from actual producer checkout and checks successful job/attempt/artifact
association. The optional ZIP transport has a disabled-by-default switch,
official storage redirect restrictions and no forwarded Authorization header.
It is restricted to Actions and is never invoked locally or by this rollout.
Missing privileges or any 403 must select a build; no new token, job permission
or attestation permission is introduced. Its receipts are not signatures.

## Stable reporting and rollout gates

Actual E2E and Unit matrix jobs now upload current-run cell receipts, including
failed or cancelled outcomes where the runner can still finish cleanup. The
always-run summaries check their plan and checkout identities, expected inventory
and completeness; missing receipt artifacts are visible, not assumed successful.
Informational performance outcomes remain informational in the shadow summary.
The existing strict publication performance gate is unchanged.

The new summary jobs have no extra Actions API permission and cannot independently
authenticate every final job result. They explicitly keep authoritative per-cell
proof and publication authorization false. They do not replace the existing
required checks. `validate_cells` is the stricter future gate:
every planned matrix cell must appear exactly once, carry the current plan,
source/tree/policy/run/attempt identities, and have actual successful execution.
Missing, skipped, stale, duplicated, cancelled or inherited acceptance fails.
The inventory must be nonempty, match reviewed policy and bind mandatory typed
identities to an external run identity. A recomputed artifact hash alone does not
establish trust. Unit and E2E have distinct inventories and run identities.

Before enabling omission, CI must run planner and proof contracts, compare shadow
plans with full runs, and validate deleted/renamed packages, lock/config changes,
Core/ABI changes, script-only repairs, test-only repairs, forks and failure paths.
Planner/verifier/policy changes themselves require full validation. A discrepancy
expands the graph or disables the affected optimization, rather than suppressing
the full-run failure. A future reviewed switch must not be controlled solely by
unreviewed PR code.

The existing nine native targets, two release-profile Docs references, three
platform functional consumers, 297 npm and 28 SDK seal, and complete ARM64
business-2048 performance requirements remain authoritative. A final candidate
and final merge tree require fresh acceptance evidence under the existing release
policy. Old SHA success is not current SHA acceptance.

Artifacts and compilation caches remain separate. Candidate scanning is limited
to three producers; missing or expired artifacts fall back to building. Shadow
plan and summary artifacts use 14-day retention. This rollout does not create
additional large target-cache writers, paid runners, credentials or retention
without bounds.

## Same-run platform chains under review

The follow-up branch separates three primary chains. Each calls the same native
build recipe with one fixed ABI, then starts its own Examples and TypeScript jobs
after that producer succeeds. These consumers no longer wait for the other two
primary producers. The producer concurrency group includes the selected platform
so the three calls cannot cancel each other. The six extended native targets and
the two independent release-profile Docs lanes remain unchanged.

| Existing functional name | Name after reusable workflow prefix |
| --- | --- |
| Examples Test (ubuntu-24.04) | Primary Linux / Examples Test (ubuntu-24.04) |
| Examples Test (macos-latest) | Primary macOS / Examples Test (macos-latest) |
| Examples Test (windows-latest) | Primary Windows / Examples Test (windows-latest) |
| Typescript Test (ubuntu-24.04) | Primary Linux / Typescript Test (ubuntu-24.04) |
| Typescript Test (macos-latest) | Primary macOS / Typescript Test (macos-latest) |
| Typescript Test (windows-latest) | Primary Windows / Typescript Test (windows-latest) |

The release verifier already requires exactly one exact or slash-prefixed match;
its required functional names remain unchanged. Native producer names gain a
platform and `native` caller prefix but retain their exact ABI suffix, which the
read-only evidence audit already matches uniquely. Existing seven branch-rule
compatibility names remain exact and require all primary chains to succeed.

The `call-rust-build-user`, `examples-test` and `ts-test` job keys remain strict
aggregate barriers for existing final consumers. Assembly still requires all nine
native targets; seal still requires complete assembly and functional evidence.
The primary aggregate now waits for the entire three-platform TS/Examples chains,
so assembly waits more conservatively than its previous native-only barrier.
Performance still waits for the existing full functional/Docs/assembly barriers
and keeps the complete business-2048 workload. Those final barriers are deliberately
conservative; this change only starts platform functional tests sooner. It neither
omits tests nor enables cross-run reuse. Actual check names, execution timing and
unchanged final gate behavior must be confirmed in CI before claiming a speedup.
