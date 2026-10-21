# Farm's pinned fervid_parser patch

Source: `https://github.com/phoenix-ru/fervid`, commit
`4f26ab48f81eb9688fb84e9fa14541373d5bca7d`, `crates/fervid_parser`.
The upstream Apache-2.0 license is retained in `LICENSE`.
The unchanged `crates/fervid/benches/fixtures/input.vue` is included under
`tests/fixtures/input.vue`; the upstream test include path is adjusted so the
parser's own test suite can run outside its original workspace layout.

This replaces the existing `fervid_parser` dependency through the root Cargo
git-source patch; it does not add a parser package or upgrade SWC. The manifest
expands the upstream workspace dependency versions and pins `fervid_core` to the
same source commit so the crate can live outside the upstream workspace.

`src/sfc.rs` wraps SWC's streaming `ParserInput` to preserve a PascalCase start
or end tag before HTML tree construction. In particular, Vue `Button`, `Input`,
and `Title` components must not become native HTML elements or trigger native
insertion/RCDATA rules. Lexer state, positions and errors are delegated, source
bytes remain unchanged, and lowercase HTML/SVG tags retain the existing path.

Both Farm's direct parser calls and fervid's internal parser calls must resolve
to this same patched crate. Keep the patch until the pinned upstream parser
contains equivalent behavior and Farm's parser, compilation, SSR and client
regressions pass against that replacement. Runtime acceptance and artifact
identities are recorded separately under `docs/v2-vue-rust-ssr-acceptance.md`.
