# First live CI native reuse scope

`ci-leaf-native.py` is an actual execution policy, separate from the generic
shadow planner. The first enabled scope is deliberately narrow: ordinary
same-repository PRs and main pushes that differ only in regular Markdown files
under `docs/experiences/`. Every other committed byte and executable mode must
match an independently read trusted main source and the original producer.
This private engineering-documentation exclusion is a repository policy. If a
new build starts reading that directory, the exclusion must be reviewed again.

First builds and unavailable evidence execute all nine native targets. A valid
same-chain GNU bundle removes only `linux-x64-gnu` from the actual native build
matrix. The other eight native targets, all three TypeScript/Examples chains,
independent release-profile Docs validation, and three-platform full
business-2048 measurements still execute. There is no inherited test success or
claim that TS/Examples selection is implemented by this first scope.

Main push is a real CI/Unit trigger. A successful GNU producer records the
complete native cohort, original member hashes, exact profile, Farm ABI, pinned
active Rust toolchain, measured Rust/Cargo/linker versions, build flags, actual
container image ID and the complete committed input fingerprint. GNU CI uses
an explicit measured `gcc` linker. Other platforms do not borrow this context.
The existing generic shadow context remains insufficient for reuse.

The existing Type Check job's `actions: read` identity resolves at most three
completed main CI runs. It independently checks the successful native producer
job and upload step, immutable artifact ID/digest, exact regular ZIP members,
every original byte, source tree, requested image and unchanged build inputs.
Release workflows/namespaces, fork producers, partial or cancelled producers,
legacy artifacts, unknown input changes and inaccessible evidence build again.
An aggregate failure may leave a valid successful producer's bytes, but grants
no acceptance inheritance: current TS/Examples and performance still run.

Consumer materialization has its own name and hash commitment. It records both
`originBuildSHA` and `consumerSHA`; it is not uploaded as a new native build.
Every consumer checks the current run/attempt/source, complete inventory and
original member hashes. Assembly retains the original producer identity. The
GNU disposition job actually loads the native binding before current tests.
The stable always-run execution gate checks the complete current needs set,
and the six existing required TS/Examples aliases depend on this gate.

The bundle is capped at 512 MiB and retained for three days; lookup is bounded
to three runs. Consumer materialization lasts seven days. This is artifact
storage, not the separate dependency/target cache budget. An absent or failed
optional baseline upload never certifies reuse and leaves native builds intact.

Local validation uses mock GitHub metadata and real ZIP/member bytes in owned
temporary directories. It covers first-full-build, exact second-leaf selection,
byte materialization/consumption, source/mode/config changes, profile/image/
toolchain mismatch, corrupted/partial data, stale main and denied access. It
does not execute Rust, a container, browser E2E, or a registry publication.
Real runner/toolchain measurements and end-to-end time savings still need CI.

General semantic suite selection, other-platform byte reuse and incremental
release remain separate unfinished work. This policy does not enable them by
setting an `executionAuthorized` field in an uploaded plan.
