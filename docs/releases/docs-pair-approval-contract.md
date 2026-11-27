# Docs 2.0.0 retained attestation

This is an offline policy evaluator with fixtures. Nothing invokes it in a
live deployment. It adds no workflow, registry writer, signer, credential
access, GitHub permission or change to the default release path.

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
