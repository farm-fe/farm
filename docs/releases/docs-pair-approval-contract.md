# Docs 2.0.0 approval contracts for review

This change provides offline policy evaluators and fixtures. Nothing invokes
them in a live deployment. There is no new workflow, registry writer, signer,
credential access, GitHub permission, or change to the default release path.

## One-run approval

The existing credential-free producer freezes the two original archives.
A future writer's preflight must independently verify the producer job,
first run attempt, artifact ID and digest and the actual bytes. It must
present that source SHA and tree, original run, artifact and both archive
digests together with the proposed writer's own run ID and attempt one.
The approval contract makes a deterministic comment binding these inputs,
the verified environment's numeric ID and intended staging tag. It accepts
only one positive, exact GitHub Environment review from an allowlisted
independent maintainer. It rejects a replay in a different run, a stale
artifact, self-review, ambiguous reviews or any automatic stable tag move.

Before integrating a writer, maintainers must review and verify the
environment name and numeric ID, protected main-only configuration, required
reviewer or team, distinct reviewer identities, no self-review or bypass and
the exact independent reviewer ID allowlist. Those values are not assumed
to exist or supplied by this draft. The staging tag in tests
(`farm-docs-bootstrap`) is a proposal, not an approved publishing policy.
Its availability, any intended stable tag and separate promotion decision
must be reviewed. Do not reveal or move the repository's existing credential.

A future writer should need read-only GitHub access, not a repository write
credential. The reviewed maintainer must own the one-shot permission across
runs: GitHub job concurrency and missing historical artifacts are not durable
proof that another run did not submit. If an npm request might have been
submitted, reconcile by reading the public package bytes. Do not approve again or
retry the upload blindly. No automatic cross-run claim or live reader is
implemented by this change.

## Retained provenance, not publication authorization

A separate future attestation verifier must verify the actual receipt bytes
with the repository's reviewed pinned CLI and a real cryptographic bundle.
This policy checks fresh verified certificate and statement output, not saved
verdict JSON. Two independently known original package hashes, sizes and
integrities must match the publicly verified retained receipt. The signed
predicate binds the exact first-attempt producer artifact ID and digest. The
signer trust tuple is distinct from the original product source and must name
the approved protected main workflow, caller, signer SHA and run. These
inputs cannot be inferred from the untrusted receipt. The policy grants no
authority to publish or promote. No signer needs an npm credential.

Any actual workflow integration, protected role and signed adoption reader
require separate review. Neither the pending version PR nor the archived
signed inventory is modified by this increment.
