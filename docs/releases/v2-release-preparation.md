# V2 candidate release preparation

This is a reviewable candidate scope, not publication approval. Run
`node scripts/v2-release-preflight.mjs` for local manifest consistency.
`--publication` deliberately fails until the evidence below has been supplied
and a separately reviewed publication preflight replaces that closed gate.
Neither command installs dependencies or publishes packages.

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
- Verify reuse of registry `farmfe_macro_cache_item@2.1.0` in an external
  consumer. Registry checksum is
  `bf179f567f72a3828f74aff319c1150cdb3335029bfc125ff65774b476e8e502`.
  Published edition 2021 differs from local 2024; published feature attributes
  and identifier formatting also differ. Generated API similarity is not a
  compatibility result. Never overwrite an existing version.
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

## Performance scope

Commerce currently has Farm configurations and Vite-plugin compatibility
adapters, not an independently verified equivalent Vite build. The first report
therefore measures Farm only. A Vite comparison requires an equivalent workload,
asset/plugin behavior and fixed toolchain before it can be called fair.
