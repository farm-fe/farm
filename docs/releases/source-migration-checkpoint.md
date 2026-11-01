# Source migration checkpoint

This draft preserves reviewed pending source from an older development checkout
on top of main. It is a migration checkpoint, not a release acceptance decision.

## Included source

- `packages/create-farm/index.js`: the existing NAPI-generated loader changes for
  Android, FreeBSD, additional Linux architectures and OpenHarmony. This preserves
  dispatch code only. The package manifest still declares nine release targets;
  this checkpoint does not add platform artifacts or promise support for them.
- `scripts/native-crash-debugger.cs`: the original Windows x64 diagnostic source,
  without its output. It references `FarmNativeStacks.ThreadTrace` and
  `FarmNativeStacks.Capture`; the companion type was not available in this source
  set. It is retained for recovery/review and is not wired into any build, test,
  or product command. Do not treat it as a standalone runnable debugger.
- `docs/patches/vuetify-*.patch` and `docs/vuetify-delivery/`: existing Vuetify
  source patches and their accompanying handoff/manifest. They are stored as
  source patches, not applied to Farm or automatically published. The complete
  patch is bound to the upstream commit recorded in its manifest. Historical
  acceptance claims in the supplied handoff were not rerun during migration.

## Equivalent changes skipped

Main already has the pending `OutputDirectoryLock` binding export and declaration,
`Compiler.flushCache`, the corrected freeze-module source map snapshot, and the
core license. Copying the older generated binding files would remove newer main
APIs, so their already-integrated additions are not replayed. Editor newline and
snapshot line-ending-only changes are not source changes.

## Retained locally

The old branch history, unresolved author-content edits, build/evidence trees,
machine-specific notes, and Sincom are outside this PR. Other ownership-blocked
worktrees were not inspected. This PR is not a complete backup of those directories;
keep the original checkout until they have a verified recoverable backup.

## Validation boundary

Migration checks cover non-patch whitespace, loader syntax and mocked platform dispatch,
patch parseability, manifest JSON and the complete-patch digest. No native
artifacts, generated declarations, Rust build, full E2E, Vuetify rebuild or live
crash-debugger execution are validated here. No workflow or publication trigger
is changed. Raw patch context lines retain their required leading spaces, so a full
git diff --check flags those stored patch lines; non-patch whitespace is clean.
