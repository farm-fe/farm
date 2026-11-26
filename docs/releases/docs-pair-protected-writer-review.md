# Manual Docs 2.0.0 bootstrap

This is a one-time publication lane for exactly `@farmfe/docs-theme-default`
and `@farmfe/docs` at 2.0.0. Normal PR review and CI apply. Do not run it from
a pull request or add an extra deployment approval or Environment.

The credential-free `docs-pair-bootstrap.yaml` is the original producer
(workflow ID 428986974). The publisher is a distinct workflow,
`.github/workflows/docs-pair-publish.yaml`. Only a maintainer's manual first
attempt on protected `farm-fe/farm` main can enter. The runtime verifies the
registered workflow paths and IDs, trusted repository, exact initiating actor,
current protected source SHA and tree, original producer run and successful
upload, immutable artifact ID and ZIP digest, and each archive's bytes and SRI.
It does not accept a previous run, a verdict file, a repacked tarball or a
different source. Registration is read from the GitHub API on the fixed path.
No workflow identity is invented.

## First run

1. The writer workflow must be reviewed and landed before the versioned main
   commit selected for the freeze. Confirm the exact protected main SHA/tree
   includes both 2.0.0 manifests. Run the existing credential-free producer
   once for that SHA/tree; check its first attempt, successful upload, artifact
   ID/digest, and both archived tar SHA-256 values. Keep the originals.
2. Before dispatch record the intended run in the maintainer release notes,
   independently check that no prior writer run or possible upload exists,
   and enter the exact source SHA/tree, producer run, artifact ID/SHA-256,
   `farm-docs-bootstrap` and `first-and-only`. This is a one-shot operation,
   not an interpretation of an npm 404. Keep the run ID and its journal.
3. A separate credential-free preflight job verifies both archives and that
   the two exact versions and the proposed staging tag are absent. The writer
   repeats all checks before reading only the reviewed repo credential in
   the single restricted upload step. GitHub permissions are Actions and
   contents read only. No Git or deploy write or identity token is requested.
4. The writer can submit each original only once and never uses another
   credential or npm write retry. Verify exact public theme tar SHA-256/SRI,
   then run a clean install with the public theme and the original frozen
   local Docs tarball to satisfy the theme peer. After that install,
   recheck the theme's exact public bytes and staging and stable tags
   before submitting Docs. Then independently verify both public exact
   tar bytes/SRI and a wholly public clean consumer install. A changed
   stable tag aborts. Token-free
   verification reads are bounded; a missing public object after a possible
   submission is not a reason to submit again.
5. The only tag this workflow can write is the fixed staging tag. Verify the
   staging targets and old stable tags as part of the result. Moving
   `latest` is a different reviewed operation after clean public pair
   verification. The final publication proof here is unsigned, so version
   preparation and ordinary full-family finalization still need separately
   reviewed signed retention and the genuine latest confirmation.

## If anything stops

Concurrency serializes runs; it is not a durable global claim. The runner
requires exactly its own first attempt in bounded same-workflow history.
It checks history and protected main again after npm setup and public
registry reads, immediately before recording and allowing each submit.
GitHub reads and the npm write are not atomic; do not move main, publish
manually, or change tags while this workflow is running. Any previous or ambiguous
run stops, even if no initial failure is logged. History may be deleted or
incomplete, and a crash can lose the local journal. Maintain one-shot
operator control independently; do not delete a run, rerun, or rely on
missing history, a missing artifact, or a 404 to recover.

For a definitive preflight-only failure, inspect the exact run and job
records and verify no publish job/step ever entered, then check both npm
versions and staging tags read-only. Do not clear history or reset the
writer: if another first attempt is genuinely needed, bring this evidence
to a separate repository review of the recovery. If a submit could have
happened, reconcile only by reading exact public bytes against the original
archives. Stop on any discrepancy or uncertainty. A new attempt is not
automatically authorized.

Tests use a synthetic runner and fake network/npm subprocess only; there
has been no live dispatch, credential check, upload or npm write from
these tests. Keep the old offline approval proofs and archived release
evidence untouched; the new workflow does not call them.
