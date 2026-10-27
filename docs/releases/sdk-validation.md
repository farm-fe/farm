# Independent stable SDK validation

The pre-publication Windows x64 consumer passed three actual release-profile
tests on 2026-10-27 using the 26 new sealed crate archives and the original
registry macro archives. It has its own workspace and no path dependencies or
workspace patches. Core cache round-trip, plugin construction and ABI 0.5.4,
Compiler/Toolkit public types and the new glob matcher are covered.

The archive source commit is `ffcb09091449d44847311ea26867572956eac385`.
This is an isolated local registry proof, **not a public registry install** and
not a claim that all features, targets or the 297 npm packages were validated.
The source preflight separately checks that Rust release inputs still match.

The metadata includes a historical `farmfe_utils 0.1.6` used only by
`enhanced-magic-string 0.1.0`. Farm's own consumers use the exact selected V2
utility. Any direct Farm binding to that old version fails verification.

`verify-sdk-consumer.mjs` rechecks the actual test log, reviewed manifest,
Cargo.lock checksums, full Cargo metadata and all 28 registry archives.
`seal-sdk-consumer.mjs` copies those verified bytes to a new portable evidence
directory, leaving the original run untouched. It includes only the selected
archives, not the entire dependency cache.

The original immutable macro archives declare MIT but omit LICENSE files.
Their embedded VCS commits are respectively
`396a228c3c8c12712f3b8fbd85ea3634d8a8a58a` and
`c35db3fcefe5c284d60e5a1d2fb70d06b2f6af84`. Both commits contain the same root
MIT text, copyright 2022 farm-fe, with no crate-local override. The unchanged
original license bytes and pinned source links are recorded in
`docs/release-evidence/published-macro-license.json` and accompany portable
evidence. No immutable registry archive is rewritten or republished.

`externalSdkVerified:true` is independent of npm archive coverage. Stable npm
publication still requires all candidate archives, legal/native provenance,
stable version state and live published Rust Core checksum/ABI checks. A public
registry consumer must be tested after the required Rust versions exist.
