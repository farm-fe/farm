# Evidence adapter implementation status

> Historical staged implementation notes below. Current live entrypoints are
> documented in `independent-incremental-chains.md` and `ci-suite-selection.md`.
> Lightweight local contracts have now run. Generic shadow output remains
> informational; it is not the authority for live byte or suite decisions.


The second batch adds `incremental-chain-evidence.py` and the actual Actions
entrypoint `incremental-chain-runtime.py`. The existing Type Check job already
has `actions: read`; it now queries a bounded same-chain baseline inventory and
uploads a current CI plan. This adds no job permissions and does not run any
transport locally. The production transport refuses a non-Actions environment.

An artifact cannot authenticate itself with `verified: true`. The adapter reads
repository, default branch, workflow run, exact attempt, jobs and immutable
artifact metadata independently through the existing GitHub identity. It checks
the artifact namespace, compressed byte digest, exact single JSON member,
producer lifetime, successful required jobs, exact source commit/tree, complete
package family, file modes, dependency closure and reviewed execution policy.
Incomplete measured toolchain/flags context is rejected. A context reader has
not yet been implemented or connected, so copying JSON measurement claims does
not enable reuse. The fixed producer job and successful upload-step interval
come from policy and independent job metadata, not a ledger-chosen job name.
Release additionally requires the successful public-consumer job and independent
public registry byte verification. The new registry reader streams original npm
and Cargo archives, checks official metadata integrity and calculates SHA-256;
ledger checksum fields are never copied back as their own evidence. This read is
Actions-only, without auth headers or redirects, with a ten-minute/three-GiB
total bound and a 100-MiB per-archive bound. If the complete verification cannot
fit those bounds, the evidence is unavailable; it is not marked verified.
Only default-branch push/dispatch executions can establish this baseline; PRs
and forks cannot certify a reusable baseline for other PRs.

Lookup is bounded to three successful runs, an exact artifact name filter and
three job pages. Denial, malformed data, expiry, missing proof or a changed
default-branch policy results in a complete build within the current chain.
No rejected local ZIP or jobs API operation is retried. The archive transport is
used only inside the authorized Actions job, never on the Windows machine.

The returned binding includes the ledger content hash, original artifact digest,
policy SHA and producer run/attempt. `recheck_binding` rejects a moved successful
baseline and must be called after acquiring the publication lock before upload.
It is a comparison primitive, not a lock implementation. No new publication
mutex or release execution entrypoint has been connected in this batch.
The adapter only returns provenance and selection data; it cannot grant upload
permission. CI/release namespace or profile mismatch is rejected before reuse.

The source-contract job runs both new Python contract files remotely. No local
test, spelling check, installation, build or package operation has been run.

## Still required before enabling omission or publication

- Actual successful baseline production, including immutable archive references,
  complete measured build contexts and cross-workflow Unit acceptance for CI.
  Existing legacy/shadow artifacts do not satisfy the new contract.
- The byte resolver must independently match current profile/recipe/toolchain,
  host/target, ABI, feature/flag and dependency inputs before materializing bytes.
  A valid baseline alone does not establish current binary compatibility.
- Dynamic scheduling, semantic suite runner result counts and the stable gate
  need real runner integration. `validate_execution_results` already rejects
  missing/duplicate cells, skipped work, zero counts and foreign plan/attempt
  identities, but is not yet called by the production summary.
- The release workflow, lock-protected baseline re-read, independent release
  build, original-byte sealing, per-package continuation and public-consumer
  ledger completion remain to be connected. The canceled full-build PR is not
  required or modified.

Both current execution and publication authorization remain false until those
checks are implemented and remotely validated. These are implementation stages,
not the final delivery state: the goal remains usable incremental CI and release
when PR #2330 is merged.
