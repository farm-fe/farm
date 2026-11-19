# Unshipped source archive

`create-farm-index.js.snapshot` is an exact byte copy of
`packages/create-farm/index.js` at Farm commit
`9760c9d6b763644e5127bf01311bdf2d3bc721ce`.

SHA-256: `130263bb5ad77eb3c396d318122025388db9f66f00f1fa57e1cfdd33bbb54e26`

This is recovery material, not executable package input. Do not import it or
copy it into a release. It preserves 13 additional native loader routes whose
platform packages are absent from the package manifest and native package tree.
Mocked dispatch tests do not establish native availability or platform support.
The shipped loader is unchanged from main. Shipping these routes requires a
separate implementation, native install acceptance and normal release review.

The Windows diagnostic remains at `scripts/native-crash-debugger.cs` for source
recovery only. It lacks its `FarmNativeStacks` companion and is not standalone
or connected to build, test or product commands. No live debugger was executed.
It can launch and terminate its own child process, contact the supplied readiness
URL and write command arguments and diagnostic output to local files. Any future
activation needs review of trusted inputs, log redaction and handle management.
Do not include secrets in its command arguments or publish its output.
