# Third-party notices

Farm Core uses the following separately installed Node.js dependencies. Their
upstream MIT license texts are included here because the observed installed
versions contain a README license declaration without a standalone license file.
The original copyright and permission notices are preserved unchanged.

| Dependency | Observed version | License text | Version source |
| --- | --- | --- | --- |
| @polka/compression | 1.0.0-next.25 | [MIT](licenses/polka-compression-MIT.txt) | [Pinned upstream package](https://github.com/lukeed/polka/blob/eba0aaff89c8b0765dcc89b23f738399c5edff5e/packages/compression/package.json) |
| sirv | 3.0.0 | [MIT](licenses/sirv-MIT.txt) | [Pinned upstream package](https://github.com/lukeed/sirv/blob/3bd2ff66ece3f44e450a48fff3ff0e41d1e56dc2/packages/sirv/package.json) |
| sirv | 3.0.2 | [MIT](licenses/sirv-MIT.txt) | [Pinned upstream package](https://github.com/lukeed/sirv/blob/1135207e92c40354543cbd15c763c7a61d79d432/packages/sirv/package.json) |

The repository-root license at each pinned commit supplies the corresponding
text. The version-to-commit association comes from that exact version's npm
registry `gitHead`, cross-checked against the pinned package manifest. These
notices do not change the dependencies' licenses or Farm's MIT license.

The listed sirv 3.0.2 license was also checked against the [root license at its exact upstream commit](https://github.com/lukeed/sirv/blob/1135207e92c40354543cbd15c763c7a61d79d432/license); it is byte-for-byte identical to the included text.

This is a partial notice inventory, not a complete list of all transitive Node
or native dependencies. Actual installed versions and the final distributed
dependency closure require a separate audit. In particular, Core's `sirv`
semver range may resolve to another version after a normal dependency install;
recheck this notice against the final lockfile and installed package.

## Editor palette reference

The native scope-to-color implementation is maintained by Farm. Its familiar editor palette references the MIT-licensed VS Code default themes; no VS Code language server or semantic-token service is bundled.

Sources consulted on 2026-10-24: https://github.com/microsoft/vscode/tree/main/extensions/theme-defaults/themes . Retained upstream license: `licenses/vscode-theme-reference.txt` (https://github.com/microsoft/vscode/blob/main/LICENSE.txt).

## Native glob compatibility

Farm's `farmfe_ecosystem_glob` derives its matcher from picomatch-rs 0.1.0,
commit `ceb39c90fc719fbbab79216a139f31c174272ed3`, with the scoped Farm fixes
recorded in that crate's `UPSTREAM.patch`. It does not embed the NAPI wrapper.
The original project MIT text and copyright are retained in
[licenses/picomatch-rs-0.1.0-MIT.txt](licenses/picomatch-rs-0.1.0-MIT.txt).
Source: https://github.com/Maidang1/picomatch-rs/tree/ceb39c90fc719fbbab79216a139f31c174272ed3 .

The matcher uses fancy-regex 0.14.0. Its original MIT license is retained in
[licenses/fancy-regex-0.14.0-MIT.txt](licenses/fancy-regex-0.14.0-MIT.txt),
from that exact crates.io package. This addition does not make the partial
inventory above a complete native license audit or resolve the separate
Tardar notice-material gap.

## Chokidar public type declarations

Farm Core distributes a type-only adaptation of chokidar 3.5.3 under dist/vendor.
The original complete MIT notice (Copyright 2012-2019 Paul Miller and Elan Shanker)
is included at [licenses/chokidar-3.5.3-MIT.txt](licenses/chokidar-3.5.3-MIT.txt).
The normal declaration producer verifies the exact upstream declaration and license
SHA256 values, preserves the upstream declaration as text, and records provenance
in dist/vendor/chokidar.provenance.json. The only adaptation removes an invalid
claim that this class implements Node's native fs.FSWatcher; no methods are added
and the installed watcher runtime is unchanged. The declaration's anymatch type
dependency is explicitly pinned to the already-used version 3.1.3; it remains a
separately installed dependency with its upstream license, rather than copied code.
