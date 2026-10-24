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
