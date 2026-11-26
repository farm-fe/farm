# Protected Docs 2.0.0 writer: review proposal, not an enabled workflow

This change adds offline handoff code and test fixtures, not a workflow,
publisher, credential access, signer, added permission, or change to ordinary
release policy. No production writer calls this code. The draft may be
reviewed independently of the source version change.

## Verified facts and fail-closed prerequisites

- The credential-free `docs-pair-bootstrap.yaml` workflow in `farm-fe/farm`
  is producer ID `428986974`. The writer path under review is distinct:
  `.github/workflows/docs-pair-publish.yaml`. The writer workflow's registered
  ID, exact running job identity and protected Environment ID and independent
  reviewer allowlist must be verified before any live integration. Missing,
  stale or ambiguous values deny. The proposal does not invent identifiers.
- Use the dedicated `docs-npm-bootstrap` GitHub Environment with only the
  `main` branch, a separate required reviewer, self-review prevention and
  administrator bypass disabled. Verify the Settings configuration and use
  the pinned numeric environment ID in the approved control policy as well as
  the live approval result. No shared Preview or Production environment;
  no new secret or move of an existing secret in this review. The proposed
  environment is not assumed to exist.
- A subsequent reviewed workflow can use `contents: read` and `actions: read`
  to verify provider records; no repository or deployment write or OIDC for
  the bootstrap writer. Use only `NPM_BYPASS_2FA_TOKEN` in the single protected
  upload step after all gates, never `NPM_TOKEN`, any other credentials, or
  repository-resident fallbacks. The old 0.0.1 first-name workflow is not a
  substitute. This draft does not test token existence, validity or scope.

## Proposed first-attempt flow

1. Use an exact reviewed protected current-main SHA and Git tree that contain
   both 2.0.0 manifests. Freeze and retain the successful first producer
   attempt, its exact upload job, artifact ID/digest, and the original ZIP with
   both tarballs. The future writer and producer are distinct registered
   workflows and runs, on this same immutable source SHA. Standalone approval
   does not authorize a source update or an artifact from another run.
2. A credential-free preflight proposes the canonical approval comment from
   the real artifact metadata and ZIP bytes. The protected writer must repeat
   the SHA/tree, actual original bytes, role and first-attempt verification
   without relying on supplied verdict files. An independent maintainer
   records which writer run consumes the one-shot permit and approves this
   exact artifact and run through GitHub. The adapter validates the live
   Environment review for that run, approver and environment, then rechecks
   that main did not move. Before reading credentials, the future caller
   must additionally bind the *actual* GitHub executing job to that reviewed
   workflow/run/attempt and enforce this rule at the upload entry point.
3. Concurrency is only serialization; use a fixed pair-wide group without
   interrupting an in-flight upload. It does not prove no other run already
   submitted. The maintainer must never approve a second run after a possible
   submission without separately resolving the first outcome. If this
   procedure is insufficient, postpone activation until there is a reviewed
   durable atomic claim scoped to the two exact versions, not repository-wide
   write access. Do not infer a new permit from missing history or an npm 404.
4. Under that exact permit, send the theme original once. Read-only poll its
   public exact package and archive digest for at most 1,200 seconds. Prove the
   public theme installs with the independently frozen local Docs tarball,
   because its required Docs 2.0.0 peer is not public yet. Then send the
   original Docs once. Verify both publicly fetched original SHA-256 values
   and SRI and a wholly public clean consumer install. An ambiguous upload
   blocks any further submit; no npm transport retries or interactive
   fallback. Log only sanitized status and archive identities.
5. `farm-docs-bootstrap` is the proposed empty staging tag, not approval to
   set it; verify the separately agreed policy and live tag state. Do not
   change `latest` before exact public pair and consumer success. A separately
   reviewed latest confirmation and signed retained adoption are required
   before ordinary full-family finalization is expected to succeed.

The immutable handoff evaluator takes injected read-only fixtures, so an
untrusted pull request cannot reach a GitHub or npm network transport through
it. Its tests exercise swapped role identities, wrong repository, replay,
truncated or failed producer jobs, substituted bytes, stale or ambiguous
approval, and main moving after approval. This is not evidence of a live run.

## Operational successor under review

The successor supplies the protected runner's actual confined GET and npm
subprocess path but leaves the runner physically disabled in source and
unreferenced from GitHub Actions. No command flag or environment variable
can turn it on. The approved writer ID, protected Environment ID and
reviewer allowlist, staging tag, executing job and reviewed pinned
Node/npm installation must be wired in a later review; nothing here
registers or activates them.

Local tests exercise the actual runner entry with fake GitHub and public
registry responses, and test subprocess arguments, identity, isolation and
clean installs with a fake npm process. They verify that auth never follows
the artifact redirect or enters npm installs, the writer aborts if an older
partial upload or a stable tag change is observed, only public reads are
retried, and the theme proof includes the undelivered frozen local Docs peer.
The real job's journal is never overwritten. Each submission intent is
flushed before the one permitted call, but a job crash can lose the local
journal, so the independent maintainer must still enforce one-shot control
across runs. The final output is unsigned retained evidence, not authority
to advance any baseline or move tags. Never treat a missing result as proof
that an upload did not happen.
