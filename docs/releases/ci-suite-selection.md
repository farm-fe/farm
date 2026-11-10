# PR suite execution scope

Every PR runs the complete Unit, TypeScript, Examples, Docs, type and candidate
archive consumer tests, including documentation-only and version-only changes.
Test execution does not query a main test baseline or inherit earlier results.
The ordinary Changesets rule remains separate and unchanged.

Only native builds may be reused. A cache hit and a cache miss have the same
complete test inventory and case-count requirements. Missing trusted build
bytes cause a current build; they never cause tests to be skipped.

The six same-run TypeScript/browser receipts are bound to fixed upload IDs,
digests and original producer attempts. The execution gate checks actual
current results, required build/assembly jobs, and complete case counts. Missing,
failed, cancelled, skipped or foreign evidence cannot satisfy the gate.
The primary chains retain create-farm tests, and the PR candidate workflow
installs and exercises the current unpublished npm and Cargo archives.

Main builds, packs and publishes from its actual fixed source. It does not wait
for a CI/Unit test ledger or require a matching PR SHA. Frozen archive identity,
registry conflicts, original-byte recovery and registry read-back remain release
checks. Performance sampling and reports remain local only.

The former reduced suite selection and main-baseline lookup have been removed.
See [final progress](v2-final-progress.md) for implementation and verification
status; the current working draft is not a completed hosted release acceptance.
