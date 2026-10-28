# Stable V2 merge publication

The current reviewed release scope is 51 primary + 246 native npm packages (297)
and the complete 28-crate Rust SDK closure. These are an explicit release plan,
not a sample. Inventory and checksums must match the reviewed source and archives.

PR #2329 merging into main starts the current sealed release DAG. Unmerged PR
updates, ordinary feature PRs and pushes to the feature branch cannot publish.
Website deployment and auto-merge are not included. Subsequent main pushes use
Changesets Action to prepare version PRs; Cargo version PR preparation uses
release-plz. Neither versioning action receives npm publication credentials.
Only actual publication uses the farm-stable-release lock. Changesets and Cargo
version PR preparation use separate concurrency groups so their pending runs
cannot displace a pending publication after the same merge.

A later changeset-release version PR may enter the same merge DAG only after its
release plan, reviewed PR-number intent/hash and complete evidence are refreshed
in that version PR. The current V2 plan and stable-preparation receipt are not a
permanent substitute for future version preparation. Automatic future regeneration
of those reviewed records is still required before that future release can pass;
it is not implemented by ordinary pnpm bump alone. Missing records fail closed.

## Ordered executable chain

1. Evidence: exact reviewed PR head, successful latest required workflows and
   individual functional jobs, including both integrated Docs lanes, all three
   Examples/TypeScript lanes, release source contracts and full candidate seal.
2. Compare PR head, actual artifact producer and final merge trees. A changed
   merge tree requires new validation; no old-head green result is substituted.
3. Download only the exact run's complete seal; verify all archive identities,
   original legal bytes, native provenance and independent packaged SDK proof.
4. Check all immutable registry collisions before the first upload. Publish or
   safely resume dependency-ordered exact Rust archive bytes. Reused historical
   macros additionally require pinned compatibility evidence and registry bytes.
5. Verify live Core/ABI and every Rust SDK checksum, then publish only the exact
   sealed npm allowlist in dependency order, never workspace-wide changeset publish.
6. A separate read-only job verifies registry archive bytes and independently
   consumes public npm/Rust packages without workspace patches or production tokens.
7. Only its exact-run proof permits the immutable tag/GitHub Release. Existing
   identical tag/Release may be reused; different provenance is never overwritten.

Performance is explicitly informational, not a publication certification. Its
continue-on-error outcome is not used as proof of functional acceptance. Required
functional jobs are checked separately even when the overall workflow is green.
No chain relies on GITHUB_TOKEN recursively triggering another workflow.

## Reviewed intent

v2-stable-release.yaml pins the SHA256 of the actual UTF-8/LF
v2-release-intent.json. The generator accepts metadata, a new output path and an
optional reviewed PR number. The intent binds the candidate plan, required
workflow/job set, ordered Cargo identities/reuse evidence, ABI and public consumer
script digest. Regenerating or editing it requires reviewing and updating the
pinned hash; it never grants publication by itself.

## Remaining real blockers

Source license audit currently stops on missing original LICENSE material for
create-farm (ISC), js-plugin-dts (ISC), and missing classified declaration/text
for js-plugin-qiankun. This change does not relabel or fabricate those licenses.
No package is silently dropped to turn the audit green.

The complete 297 npm + 28 Rust portable seal and exact final-source consumer
receipts must actually be produced by CI. They are not supplied by unit fixtures
or by the previous local SDK probe alone. The latest source changes have not been
locally tested per user instruction; the PR push requests CI validation.

Existing NPM_TOKEN/CARGO_REGISTRY_TOKEN availability, main protection/review,
registry propagation and the final registry consumer remain publication gates.
No credentials or repository permissions are created here. A known Windows
taskkill denial still blocks claims of successful business-2048 timeout-tree
cleanup; informational performance must report failure rather than invent samples.
