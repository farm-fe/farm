---
"@farmfe/plugin-tools": patch
"create-farm-plugin": patch
"@farmfe/plugin-auto-import": patch
"@farmfe/plugin-compress": patch
"@farmfe/plugin-dsv": patch
"@farmfe/plugin-dts": patch
"@farmfe/plugin-icons": patch
"@farmfe/plugin-image": patch
"@farmfe/plugin-mdx": patch
"@farmfe/plugin-modular-import": patch
"@farmfe/plugin-react": patch
"@farmfe/plugin-react-components": patch
"@farmfe/plugin-replace-dirname": patch
"@farmfe/plugin-sass": patch
"@farmfe/plugin-strip": patch
"@farmfe/plugin-svgr": patch
"@farmfe/plugin-url": patch
"@farmfe/plugin-virtual": patch
"@farmfe/plugin-wasm": patch
"@farmfe/plugin-worker": patch
"@farmfe/plugin-yaml": patch
---

Fix Rust plugin build scripts and tooling compatibility with napi-rs CLI option parsing.

- Add backward compatibility in plugin-tools by stripping legacy `--cargo-name` before invoking `@napi-rs/cli`.
- Update Rust plugin build scripts to use `-p <cargo-package>` without the unsupported `--cargo-name` flag.
- Fix `@farmfe/plugin-react` `build:publish` script to target the correct Cargo package.
- Update create-farm-plugin Rust template and docs examples to generate the corrected build command.
