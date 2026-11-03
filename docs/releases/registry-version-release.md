# Fixed-main registry-version release

Implementation under local validation; not yet pushed or deployed.

## Trigger and selection

The npm and Cargo source workflows accept a normal push of a fixed SHA to main.
The source gate binds the event SHA, checkout tree and workflow SHA/ref to that
same repository main commit. A PR number, title, branch name or historical PR
head is not publication authorization. Squash commits are ordinary fixed sources.

Pending Changesets (or prerelease state) defer publication. The existing
Changesets Action prepares its version PR without publication credentials. Once
that PR merges, the new main event evaluates the updated versions. The absence
of Changesets is readiness to query registries, not proof that publication is
needed.

For each complete-family package, query its exact name/version. Only an explicit
registry 404 means absent. Authentication, rate limits, timeouts, malformed
metadata, redirects, identity mismatches and checksum failures abort selection.
Already published versions retain their verified public archive identity and
are not uploaded again. npm internal runtime, optional and peer dependency
ranges must accept the selected complete-family versions; published packages use
their registry declarations for this check. Unsupported range syntax fails
closed. Candidate consumer installation remains mandatory.

npm uses the complete source-validated candidate inventory, including every
supported native payload (currently 51 primary packages and 246 payloads). The
existing independently maintained `@farmfe/js-plugin-copy` exclusion is unchanged. Cargo derives its SDK family and current versions
from the fixed Cargo dependency graph (currently the 28-crate SDK closure),
independently of old npm candidate metadata. Cargo source publication remains a separate chain; it neither waits
for npm native outputs nor requires a new npm version PR when no Changesets are
pending.

## Build versus upload

Missing exact npm versions form the upload set. The shared npm/Cargo/custom
repository graph computes its preparation dependency closure. Dependencies may
need building even when their own versions are already published; they must not
be packed as new publication candidates. Selected products build from the fixed
source using the formal recipe. No historical binary or PR artifact replaces
that source build. Ordinary Cargo dependency caches remain permitted.

Changesets performs version changes before this stage. Registry selection never
invents another version bump. Code diff remains applicable to CI tests and
artifact compatibility, not the upload decision. With no missing versions the
release is a no-op; without any successful publication baseline, missing versions
can still form a complete first release.

Cargo packages selected missing crates in dependency order using normal locked,
offline `cargo package` verification. Published prerequisites are fetched as
source dependencies and their exact archive checksums are retained. No
`--no-verify` exemption is introduced.

## Locking and recovery

The existing concurrency groups serialize each chain independently: npm uses
`farm-independent-release`, while Cargo uses `farm-stable-release`. Cargo does not
wait for npm native builds. npm also retains its durable publication lock/fence
and compare-and-swap state pointer. The pointer is a concurrency condition, not a requirement for historical
PR provenance or a successful binary baseline. Under the lock, revalidate retained
registry versions and the frozen plan; preflight every selected version for
absence or exact matching frozen bytes before the first upload.

A transaction freezes its original selected set and archives. Partial completion
never causes it to regenerate already selected bytes from a fresh registry query.
Retry uses the original producer attempt, source, manifest, archive hashes and
per-package state; matching published bytes are accepted, conflicting bytes fail.
Missing or expired frozen archives after possible publication are a blocker.
The full-family checkpoint labels retained registry members as registry evidence,
without inventing a source SHA or producer run for those historical bytes.

For the old #2330 squash failure, first establish that the source gate failed
before any freeze/upload. A later main commit containing this control fix may
produce a new transaction from its own actual source SHA. It must not label that
build as the old squash SHA. If an old transaction already froze or published,
recover its original bytes and identity; do not replace them with a new build.

## Local acceptance and operational limits

Tests cover exact-version deltas, missing baseline, published dependencies,
registry errors, changed state pointers, frozen partial selections, source/tree
mismatches, squash without PR history, pending Changesets, and Cargo version
updates with stale npm candidate metadata. Existing archive/fence/restore tests
remain required. New tests run in the CI release-contract job.

Mac workflow commands are exported from the current YAML and executed in the
isolated cows worktree. Windows/Linux and actual GitHub security/artifact behavior
are not established by that local execution. Registry publication, tags, Releases
and dispatches are not exercised locally; publication tests use isolated fixtures.
A branch push is for CI only. Merging these workflows into main can publish missing
versions and requires separate user action and working existing registry secrets.
