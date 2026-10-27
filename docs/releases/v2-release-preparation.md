# V2 candidate release preparation

This is a reviewable candidate scope, not publication approval. Run
`node scripts/v2-release-preflight.mjs` for local manifest consistency.
`--candidate-seal /path/to/receipt.json` inspects actual archives and reports
missing/corrupt packages, legal byte drift, entries and native platform provenance.
`--publication --candidate-seal ... --sdk-proof ...` additionally checks stable
version state and independent SDK evidence before a live Rust registry checksum
and ABI check. It can pass only when those concrete prerequisites pass.
Neither preflight command publishes packages; source-only mode does not access
the network.

Regenerate `v2-candidate.json` from fresh offline Cargo metadata using
`node scripts/v2-release-inventory.mjs metadata.json docs/releases/v2-candidate.json`.
Review the diff; never regenerate merely to hide unexplained drift.

## Scope and order

The candidate contains 51 npm primary packages and 246 platform payloads (297).
The previous 296 omitted `create-farm-plugin@2.0.0`; 284 omitted twelve primary
packages from that older 296 scope. `@farmfe/js-plugin-copy@0.0.4` remains an
independent, unmodified release outside this candidate. Vuetify is **0.0.2**.
The JSON lists each exact name, version, owner and platform. Native payload
versions are projected from their owner using the normal packaging convention;
source platform manifests are not themselves proof of a prepared archive.

The selected Cargo SDK roots are Core, Compiler, Toolkit and ecosystem glob.
Their conservative local normal/build dependency closure contains 28 crates;
this is a verification set, not an instruction to republish every existing version.
Core 2.2.0 requires glob 0.1.0, utils 2.1.1 and macro_cache_item 2.1.0.
Existing independent 0.x crate versions are retained; no Rust V3 is introduced.

Publish Rust prerequisites before Core/SDK, native npm payloads before wrappers,
then Runtime/Core/CLI/plugins/generators. Activate Docs and theme together and
verify the independent registry consumer before the website. Stable packages,
website deployment and release tag require the final release decision.

## Evidence still required

- Bind the final Git SHA and inventory digest to CI artifact producer IDs,
  archive SHA-256/SRI, native ABI 0.5.4, platform and complete package contents.
  Reuse matching CI artifacts; do not infer whole-package equivalence from
  unchanged Rust alone.
- Re-run license/NOTICE closure against those actual redistribution archives.
  Wax/Tardar removal was verified in source dependency graphs and selected
  Windows distributions; this is avoidance, not acquisition of a missing license.
  It does not certify every final platform archive.
- Registry `farmfe_macro_cache_item@2.1.0` was tested in an isolated registry-only
  consumer with the current Core `Cacheable` trait, rkyv 0.8.16 and downcast-rs
  2.0.2. Struct and enum round trips pass (2/2) on nightly-2026-04-10,
  Windows x64. Reproducible fixture and receipt are in
  `scripts/release-fixtures/macro-consumer` and
  `docs/release-evidence/published-macro-compat.json`. Registry checksum is
  `bf179f567f72a3828f74aff319c1150cdb3335029bfc125ff65774b476e8e502`.
  Published edition 2021 differs from local 2024; published feature attributes
  and identifier formatting also differ. The actual limited contract probe
  replaces the previous API-similarity assumption; it is not full external
  Compiler/Toolkit verification. Never overwrite an existing version.
- Check every selected version against the public registry at release time,
  record existing-version compatibility and install the released SDK from a
  clean external consumer. Source manifest text is not registry evidence.
- Verify required review/protection, final CI, the approved registry-only Docs
  consumer lock/SRI receipt, and a concrete previous website deployment ID.
  Immutable package versions cannot be rolled back by overwriting them; use a
  new corrective version or an explicitly approved dist-tag action.

The existing source Vercel route intentionally refuses deployment. Keep the
current production site until the registry consumer and rollback target are
verified. No Cargo source build is permitted in website deployment CI.

## Stable version and publication entry

The one-time local stable preparation has exited beta and consumed all 46
physical changesets (31 already recorded in prerelease state, 15 outstanding).
Earlier source commits had already prepared first-stable versions, so blindly
applying the ordinary calculated release plan would bump those versions again.
`prepare-v2-stable-versions.mjs` freezes the reviewed 51 primary versions and
uses the official Changesets apply API to preserve their notes and consume the
changesets. Existing stable notes remain under an earlier-preparation heading,
without duplicate release headers. Versions include Core 2.0.0, Vuetify 0.0.2
and Docs/theme 0.0.1; Rust SDK remains Core/Compiler/Toolkit 2.2.0 and ABI 0.5.4.
The lockfile was regenerated offline; Cargo versions were already prepared and
were not arbitrarily bumped. See `docs/release-evidence/frozen-stable-versions.json`.

The requested merge-to-stable chain is being wired separately from normal
Changesets version-PR preparation. It must bind the exact PR head, actual test
merge commit and final merged tree, sharing one publication lock. A full tested
tree equality proof is required when reusing PR evidence; the chain must not
wait for main checks that are never scheduled. Until the complete sealed
artifact producer, reviewed intent and independent SDK receipts are present,
this remains an incomplete release gate, not a claim of automatic readiness.

Publication requires a named candidate artifact/run, not a new Rust build. The
workflow uses explicit pending/empty/error Changesets state, and parse or file
errors fail rather than being mistaken for empty state. Both it and the direct
`scripts/release.mjs` entry invoke the real publication preflight. The publisher
uses only verified sealed tarballs, platform payloads first, then normal runtime
dependency order. It does not rebuild source, invoke broad workspace publishing,
or change persistent npm access settings. Package scripts cannot rebuild an
already verified archive during publication. No release action has been run.

## Performance scope

The informational three-platform job now includes same-runner base/head sampling
when compatible exact-base artifacts exist, comparable retained history, a
separate real Vite 5.2.8 catalog/quote subset, and a bounded aggregate for a
trusted sticky PR reporter. A local diagnostic verified both tools' build/dev
and browser subset behavior; it is not a three-platform performance result or
full Rust Commerce equivalence. Fresh CI must exercise the new integration.
