> Current policy: performance sampling, comparisons and automatic reports run locally only. CI and release do not require performance artifacts. Commerce business correctness runs in ordinary Examples E2E with the four-category `business-smoke` fixture. Suite coverage and source/attempt evidence remain required in `Record verified CI execution baseline`. Historical measurements below are not current CI or publication requirements.

# Incremental CI and Release handoff

Snapshot: 2026-11-01. PR [#2330](https://github.com/farm-fe/farm/pull/2330)
remains Draft against `main`. Continue on
`codex/ci-incremental-followup-20261030`; implementation commit:
`3b238a2bc4e24a0cd9615e68b0b28d6fc8e0d158`.

## Architecture and enabled scope

There are two chains, sharing impact rules but never build artifacts:

- CI: PR incremental builds, tests, E2E and CI-profile Docs quality checks.
  Suite selection is enabled only for audited React/Babel `src` leaves with
  a trusted baseline. Native reuse across runs is limited to GNU targets under
  the audited docs/experiences Markdown conditions. Unknown inputs or missing
  proof fall back to full execution within CI. The broader proposed execution
  DAG remains shadow; repository-wide precise incremental execution is not
  complete.
- Release: fixed trusted `main` SHA, independent incremental rebuild/version/
  publication plan. No release baseline means an independent full rebuild.
  Candidate sealing, SDK and platform consumers are stages inside this chain.
  `publish=false` is its dry run; publication requires explicit authorization.
  The additional PR release-reference chain has been removed.

See [chain design](independent-incremental-chains.md),
[suite scope](ci-suite-selection.md) and [native scope](ci-leaf-native.md).
Same-run retries preserve each actual producer attempt, artifact name and bytes.
A failed, skipped, queued or not yet started newer job cannot inherit older success.
Carried jobs require successful status and matching job identity/timestamps.
Materialization is reuse of original bytes, not new test acceptance.

## Evidence and open acceptance

The implementation repair was locally checked with 39 lightweight suites:
343 passed, zero skipped, on Node 24.19 and Python 3.13. A Python 3.12.14
compatibility subset passed 166 tests, zero skipped. Node 22.20 was not
available locally; do not describe that version as locally verified.
CSpell covered 42 files; six YAML contracts, 35 script syntax checks and 33
bounded protected-file hashes passed. These are historical repair results,
not a claim that this handoff ran the full suite again.

For implementation commit `3b238a2bc`:

- [Unit](https://github.com/farm-fe/farm/actions/runs/41214880371),
  [Lint](https://github.com/farm-fe/farm/actions/runs/41214880382),
  [Spell](https://github.com/farm-fe/farm/actions/runs/41214880370) and
  [PR title](https://github.com/farm-fe/farm/actions/runs/41214880369) passed.
- [E2E run 41214880385](https://github.com/farm-fe/farm/actions/runs/41214880385)
  failed. The first 30 jobs were reported successful, but the failing jobs on
  later pages have not been established from accessible evidence. Do not infer
  the failing step, call CI green, or declare M5 complete.
- Actual mixed-attempt suite/performance/materialization acceptance on the
  new HEAD remains unverified. Local mock contracts do not prove hosted
  execution, remote publication locks, registry writes or recovery.
- [Vercel preview](https://vercel.com/brightwus-projects/farm-docs/87F6sCdvabHtkHiLTs3KU3YJY8gn)
  failed during its install command. Actual logs show the retired
  `farm-docs-vercel-build.mjs --install` entry intentionally exits with
  `FARM_DOCS_SOURCE_DEPLOYMENT_RETIRED`, before dependency installation or
  compilation. Its static unpublished-package message is not a registry query.
  Do not restore a source-build fallback or redeploy to conceal this guard.

No merge, real publication or manual dispatch is part of this handoff.

## Continue on another machine

Clone `https://github.com/farm-fe/farm.git`, then check out the existing
`codex/ci-incremental-followup-20261030` branch. Read `AGENTS.md` and relevant
`.agents/skills` first. The former machine used a linked worktree; copying
that worktree's `.git` pointer alone is not a portable repository.
Do not copy local credential/config files, logs, caches or Python bytecode.

1. Read current remote PR/head and CI status: this documentation push can
   trigger a newer ordinary PR CI run. Do not attribute old-SHA success to it.
2. Obtain permitted later-page E2E job evidence and the actual failing steps.
   Respect prior access restrictions; do not route around denied jobs/artifact
   reads. Keep lack of evidence explicit if access is still unavailable.
3. Reproduce a confirmed defect with light contracts, review any fix, then
   push normally. Do not blindly rerun costly native producers.
4. Before testing a legal retry, enumerate actual producer/consumer jobs and
   attempts for that fixed HEAD. Check plan, suite, performance and native
   materialization artifact names/source/profile bindings. Retry the smallest
   proven leaf/gate subgraph; do not rename old receipts or rerun native merely
   to produce aggregation evidence.
5. Retain Draft until hosted acceptance is established. Formal release remains
   a separately authorized trusted-main action. Before the eventual merge,
   remind the user about the deferred domestic/overseas sponsorship options
   (including WeChat and Alipay); sponsorship is not implemented here.

Representative lightweight contracts, after preparing the repository's normal
Node/Python dependencies on the new machine:

```sh
node --test scripts/release-chain-boundaries.test.mjs scripts/ci-suite-rerun.test.mjs
python scripts/ci-rerun-dag.test.py
python scripts/ci-restore-depth.test.py
node --test scripts/performance-attempts.test.mjs
python scripts/incremental-release-functional.test.py
```

Use the workflow-pinned Node version for hosted parity and record actual runtime
versions and skips. These commands are scoped contracts, not native builds,
performance sampling, browser E2E or the complete release acceptance gate.
