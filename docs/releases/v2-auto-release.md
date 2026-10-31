# Stable V2 merge publication

The current reviewed release scope is 51 primary + 246 native npm packages (297)
and the complete 28-crate Rust SDK closure. These are an explicit release plan,
not a sample. Inventory and checksums must match the reviewed source and archives.

PR #2329 merging into main starts the current sealed release DAG. Unmerged PR
updates, ordinary feature PRs and pushes to the feature branch cannot publish.
Website deployment and auto-merge are not included. Subsequent main pushes first
run release-plz version preparation. Its successful completion schedules the
Changesets preparation workflow on trusted main, without relying on token pushes
to trigger another run. Neither preparation workflow receives publication credentials.
Only actual publication uses the farm-stable-release lock. Changesets and Cargo
version PR preparation use separate concurrency groups so their pending runs
cannot displace a pending publication after the same merge.

A later changeset-release version PR may enter the same merge DAG only after its
release plan, reviewed PR-number intent/hash and complete evidence are refreshed
in that version PR. The current V2 plan and stable-preparation receipt are not a
permanent substitute for future version preparation. The version action records
the official Changesets plan before consuming it, then refreshes the exact version
PR's inventory, receipt and intent. The full family remains visible, while only
the changed publication delta is packed. Unchanged immutable versions require
registry checksums; a JS-only patch does not republish Core, Rust or native packages.
Explicit validation dispatch uses the approved version job's existing token with
job-scoped `actions:write`. Only the release-request and type-check jobs also
receive `actions:read` for producer evidence. Workflow-global permissions remain
unchanged and no new credential is created. Missing or denied evidence/dispatch
still fails closed; permission configuration alone is not execution evidence.

## Joint future Rust and npm preparation

An open same-repository release-plz proposal must match the current main base and
the configured workflow bot identity. The producer uploads its exact run/attempt,
source SHA, workflow path and proposal head as an immutable artifact. Its bounded
PR-body record is only an index containing artifact ID/digest and the raw receipt
hash. Consumers read that artifact from the exact run, verify its immutable API
identity, digest, member set and receipt bytes, then compare proposal/run data.
An expired/missing artifact or denied read stops preparation without another
credential or mutable-body fallback. Retention is 90 days; refresh an expired
proposal instead of reusing it. Inspection and application also prove that the
base is an ancestor of the proposal head; the PR base field alone is insufficient.
A workflow-run trigger must match that producer.

The official action's valid no-proposal output (`pr={}`, `prs_created=false`)
is an explicit no-op, allowing pending JS Changesets to continue. Updated existing
proposals returned by the action are recorded too. Inconsistent flags, malformed
objects or untrusted branches fail rather than being silently skipped. A proposal
whose exact bot/repository/branch/head and SDK version delta already match the
receipt reviewed into main is treated as consumed, even if its old Cargo PR is
still open. A moved head or unmatched version is not ignored; a new JS-only
Changeset can proceed without replaying the consumed Cargo proposal.

The selected SDK's manifests, Cargo lock and changelogs are accepted only within
the pinned proposal. The actual Cargo path reverse-dependency closure also permits
dependency-version updates in affected native consumers such as node/react. Their
package versions, paths, features and other metadata remain untouched; they are
not added to the Rust publish set. These generated manifest updates have separate
before/after hashes and are checked again before sealing. Native-owner impact
uses the same closure, including build/test constraints needed by the workspace.
Stable numeric constraint updates preserve exact/caret/tilde prefixes. Unrelated
consumer edits, source/dependency rewrites, stale proposals, multiple proposals,
SDK closure changes and version downgrades stop preparation.

The Changesets version command copies that exact proposal patch into the version
PR, records its base/head/file set, and creates a scoped native-consumer Changeset.
Cargo dependency reachability determines the affected npm native owners; Core is
always included for an SDK release. An ABI change requires the entire Farm ABI
cohort to advance. create-farm's independent Node-API-4 boundary is excluded from
an ABI-only cohort, but still advances when its actual Cargo dependencies change.
Official Changesets generates the owner versions and changelogs. A subsequent
synchronization updates their platform manifests and exact optional dependencies,
then pnpm regenerates the lock before inventory and hashes are taken.

The final version PR verifier reads the pinned original Cargo proposal again and
compares its exact contents, version delta and required consumers. It does not
accept a blanket Cargo-path exception. The separate Cargo proposal is an input,
not another PR that must first be merged or a publication trigger. The single
Changesets version PR owns the combined tested tree, release identity and sealed
delta. These new paths await CI execution; the first stable path remains separate.

## Ordered executable chain

1. Evidence: exact reviewed PR head, successful latest required workflows and
   individual functional jobs, including both integrated Docs lanes, all three
   Examples/TypeScript lanes, release source contracts, full candidate seal and
   all three candidate registry-consumer lanes. Three valid Farm-only performance
   reports and their aggregate are required for publication. Slow timings have no
   uncalibrated regression threshold: completeness, correct source and successful
   cleanup are the gates. PR performance jobs may remain informational, but their
   actual per-job conclusions and bounded strict-schema summary artifact from the
   same CI run/attempt must pass; overall workflow success alone is insufficient.
2. Compare PR head, actual artifact producer and final merge trees. A changed
   merge tree requires new validation; no old-head green result is substituted.
3. Download only the exact run's complete seal; verify all archive identities,
   original legal bytes, native provenance and independent packaged SDK proof.
4. Check all immutable registry collisions before the first upload. Publish or
   safely resume dependency-ordered exact Rust archive bytes. Reused historical
   macros additionally require pinned compatibility evidence and registry bytes.
5. Verify live Core/ABI and every Rust SDK checksum, then publish only the exact
   sealed npm allowlist in dependency order, never workspace-wide changeset publish.
   Publish archives under release-version tags before reconciling latest. Read
   latest immediately before changing it and read it back afterwards; an already
   higher version is preserved. Immutable reuse still requires byte-identical
   archives, not only a matching version number. npm and Rust keep explicit
   pending/success/failure states for every selected package in generated
   `.release-publication-status/` diagnostics. The shared publication lock prevents
   concurrent Farm release runs; npm offers no compare-and-swap for external tag
   writers, so external manual publication must stay coordinated with this chain.
6. A separate read-only job verifies registry archive bytes and independently
   consumes public npm/Rust packages without workspace patches or production tokens.
7. Only its exact-run proof permits the immutable tag/GitHub Release. Existing
   identical tag/Release may be reused; different provenance is never overwritten.

Timing comparisons are informational, not a performance certification. Complete,
valid Farm-only evidence is nevertheless required for publication. A job configured
with continue-on-error cannot substitute for its actual conclusion and strict
report receipt. Functional acceptance and the candidate consumers are separate
required evidence even when the overall workflow is green.
No chain relies on GITHUB_TOKEN recursively triggering another workflow.

## Reviewed intent

v2-stable-release.yaml pins the SHA256 of the actual UTF-8/LF
v2-release-intent.json. The generator accepts metadata, a new output path and an
optional reviewed PR number. The intent binds the candidate plan, required
workflow/job set, ordered Cargo identities/reuse evidence, ABI and public consumer
script digest. Regenerating or editing it requires reviewing and updating the
pinned hash; it never grants publication by itself.

## Remaining real blockers

The owner confirmed that the create-farm and JS DTS ISC declarations were errors:
their Farm-owned code is MIT. These packages and qiankun carry the actual Farm
MIT text. Fixed upstream Tauri, qmhc and TMQ license bytes remain separate, with
component scopes and source hashes. These direct-source records are not a claim
that every transitive dependency or native archive has completed license review.
The final seal must verify the actual legal archive members. No package is
silently dropped to turn the audit green.

The complete 297 npm + 28 Rust portable seal and exact final-source consumer
receipts must actually be produced by CI. They are not supplied by unit fixtures
or by the previous local SDK probe alone. The latest source changes have not been
locally tested per user instruction; the PR push requests CI validation.

Existing NPM_TOKEN/CARGO_REGISTRY_TOKEN availability, main protection/review,
registry propagation and the final registry consumer remain publication gates.
No credentials or repository permissions are created here. A known Windows
taskkill denial still blocks claims of successful business-2048 timeout-tree
cleanup; informational performance must report failure rather than invent samples.
