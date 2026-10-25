# @farmfe/utils

Shared Node tooling utilities used by Farm packages. The root entry exports terminal color helpers and `createSpawnCmd`; the `@farmfe/utils/colors` subpath exposes color helpers. Consult the shipped declarations for individual exports.

`createSpawnCmd(directory, stdio?)` constructs a shell-based command runner. Commands and arguments must be trusted tooling inputs; do not pass untrusted user strings. This helper is not the Farm Docs content compiler and must not be used to delegate core compilation to another build tool.

The package is intended for Node tooling, not direct browser imports. Farm Docs' final isolated package consumer tests remain separate from successful workspace builds.

Farm code is covered by LICENSE. The existing terminal-color source retains a separate ISC copyright and permission notice; THIRD_PARTY_NOTICES.md and licenses/color-ISC.txt preserve that text for distribution.
