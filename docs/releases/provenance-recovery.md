# Release provenance and retry boundaries

Only main push runs in the main repository, including retries of that same run,
may produce the isolated release signatures. PR and dispatch runs do not sign.
The signer and its exact caller templates are pinned to a reviewed Git commit.
The producer runner and executed dependencies are trusted; this is not isolation
against a compromised producer host.

A frozen signature authenticates original bytes. It does not authenticate a later
attempt's test results. Partial publication must restore the same artifact IDs,
digests, source and original producer attempt; unsigned historical frozen bytes
cannot be repacked or retrospectively certified.

## Supported recovery

- Cargo upload succeeded but signing failed: retry the failed signing job and
  its dependents. Carried upload outputs retain their original attempt. The
  protected upload action outcome distinguishes them from restore-script outputs.
- Cargo restored an earlier transaction: independently verify its original signed
  archive before signing a new binding. Missing proof fails closed.
- npm publication was interrupted: restore the original signed seal and candidate
  archives; recheck current candidate consumers, fixed-main functional evidence,
  publication ownership and registry versions before resuming missing uploads.
- Public signing succeeded but finalization failed: re-run
  **sign-public-acceptance and its dependent finalize job**, not only the failed
  finalize job. A carried old signature is deliberately not current public acceptance.
  Local tests model the new envelope; hosted rerun semantics require remote validation.
- The transaction is already complete: after current frozen-byte preflight, read
  the original tag, Release checkpoint and all checkpoint package checksums from
  the registry. Matching results permit only an already-complete no-op. This does
  not authorize new writes or replace current public tests for an unfinished
  transaction. Missing or conflicting bytes fail closed.

The code does not automatically invoke retry operations. The pending-main-evidence
controller returns a verified run/attempt/source recommendation without POST.
A maintainer must recheck current state before acting; a recommendation cannot
authorize a later attempt.
A workflow dispatch may read existing proof where supported; it cannot mint a new
signature or turn unsigned data into publication authorization.

## Local validation limits

Local tests execute production selection, byte validation and recovery algorithms
against explicit simulated API/registry/signature transports. They do not prove
hosted OIDC, GitHub scheduling, actual registry writes or non-Mac binary execution.
The production verifier still invokes the pinned CLI on its supported hosted
Linux consumer; externally supplied verification JSON is not accepted.
