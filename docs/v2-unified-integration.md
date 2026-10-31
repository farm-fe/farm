# Unified v2 candidate integration

This is an uncommitted local candidate based on main `2000ef8a790a1b565d1687cfe70564270a826e15`. Existing source, source18, preview branches and PR #2326 remain intact. No old validation is promoted to this candidate.

## Product provenance

Selected shared product paths come from `8bb3a02d8d392ff51181cc5620935dc43fdb8f25`. The two cache-writer files match `16072f08e6989f038ac673b27e38374849f6c92b`. Selected later preview fixes retain JS/CSS comments, correct test graph locks, cover ESM/hybrid classification and canonical temporary roots, and improve CLI error reporting. The original cached-worker entry registration is retained; the independent preview slice omitted it and must not overwrite it. Generated bindings, compiler snapshots and remaining template differences are still under individual review. QA logs/screenshots are not product input.

## Candidate versions and compatibility

The 24 previously 3.0.0 owned crates and internal constraints now target candidate 2.2.0. The root lock was regenerated offline and full `cargo metadata --locked --offline` passed. Third-party name/version sets did not change. Remaining owned crate registry identities were read separately: changed candidate versions are not occupied; unchanged macro crates retain their existing versions. No publication is authorized by this version edit.

Rust plugin scaffolding pins Core =2.2.0 and macro_plugin =2.0.0. The historical template Cargo.lock (Core 0.7.0) remains as an original file but is no longer copied into generated Rust projects. Registry installation of the new template remains blocked until its exact Core candidate is formally published. Tauri/third-party package versions are not altered.

Core ABI 0.5.2 and cache format 0.9.5 already supersede main's 0.5.1/0.9.1. Cache-writer synchronization changes do not change the CachedModule schema; JS comment relocation does not change exported Core layout. This is not a certificate that arbitrary old Rust binaries are compatible: native plugins must be rebuilt and tested against this exact source and pinned Rust toolchain. Old performance results do not transfer.

## Merge/publication gate

`.github/workflows/release-plz.yaml` runs on every push to main, with separate `release` and `release-pr` jobs. Neither job declares a GitHub environment approval gate or needs the final test jobs. `release-plz.toml` enables workspace releases, disables selected plugin packages, disables GitHub releases, and does not override release_always or publish. Upstream documents release_always=true and publish=true as defaults; eligible unpublished versions can therefore be cargo-published on an ordinary merge, not only on a release PR. Existing published versions are skipped, disabled packages are not released. Git tags can still be created even with git_release_enable=false. Token availability and external repository policy have not been assumed.

References: https://release-plz.dev/docs/config#the-release_always-field and https://release-plz.dev/docs/usage/release .

User approval to merge after all final-head CI and matched local docs preview is conditional. The user has now decided: merge only after final-head CI and local preview pass; do not publish until the formal version is ready. Both real publication workflows are now workflow_dispatch-only in this candidate. Rust publication requires explicit operation=release on main; its default is release-pr. npm publication requires confirm_release=true on main. Ordinary main push cannot trigger either workflow. Existing permissions, credentials, action steps and callable native builders are unchanged. Parsed actual YAML and event/ref/input simulations passed 25 checks; no remote event was triggered. Final CI/preview acceptance is still pending; no merge or publication has occurred.

## Executable validation order

1. Single-job Core mutable cache concurrency tests, using the existing main target cache but the new candidate source; save new output and exit status.
2. After resource recheck, toolkit comment tests, script module-system tests and compiler temporary-root/worker regressions; no snapshot-update flags.
3. Reconcile remaining generated bindings/snapshot/template and JS lock prerequisites; run affected type/lint/JS tests.
4. Build a matched Core/native plugin set serially, then build and serve the website locally; validate actual bilingual routes, resources, NOTICE and 404 against final candidate inputs.
5. Final reviewed docs integration head must pass all CI. Old-head greens do not satisfy this condition.
6. Website deployment consumes only a separately approved, published release set with frozen registry lock and precompiled native packages. Missing releases fail clearly. No Cargo/rustup/workspace fallback, no retry of the 45-minute online source route.

## Latest local progress

New-candidate Core build exited 0: four basic cache tests plus four separate controlled overlap tests passed (8 total). The initially narrow filter omitted the overlap module; it was then run from the same newly compiled executable, without rebuilding. Toolkit comment regressions passed 3/3, module-system integration passed 2/2, snapshot updates disabled. Website-specific layer applied from preview3113 using 585 selected paths; online-source deployment scripts/config excluded. Its root JS setup/lock and final matching website build remain pending.
