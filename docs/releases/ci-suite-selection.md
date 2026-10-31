# Current CI suite execution scope

The same-run type-check job verifies a completed main CI baseline and original
six suite receipts with the existing read-only Actions identity. Its current
selection file is bound by a raw-byte hash in job outputs to the current source,
Git tree, repository, run and attempt. Consumers independently reconstruct the
selection from committed old/new inputs before executing it.

Only source leaves under `examples/react/src` and `js-plugins/babel/src` may
select a smaller set. Source/config/manifest changes outside those audited
boundaries, unresolved workspace or npm aliases, ambiguous relevant dependencies,
policy changes, missing or inaccessible baselines and forks execute all suites.
The PostCSS test's dependency on the vite-adapter-vue Example manifest is explicit.
This is not a claim that all dynamic imports in the repository have been mapped.

TS passes the exact selected file filters to Vitest and checks actual file and
assertion results. Examples record only IDs whose build commands completed;
missing manifests/directories do not establish coverage. Browser receipts include
that build evidence and actual terminal cases. A planned empty collection is
`not-applicable` with zero executed cases, not a successful empty test process.
The stable gate checks all six current-run receipt sets and counts; skipped,
missing, failed or foreign receipts cannot satisfy it. Create-farm candidates,
Docs release-profile reference checks and full three-platform business-2048
performance remain separate and still run.

A reduced execution ledger is never accepted as a complete baseline. Lookup is
bounded to three main runs and falls back to all current suites when no complete
baseline can be independently verified. No prior case success is inherited.
CI build bytes and release build bytes use separate chains and are never mixed.
