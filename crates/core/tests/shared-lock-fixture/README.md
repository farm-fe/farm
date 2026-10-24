# Dynamic plugin lock regression

This isolated workspace builds a DLL and a host executable against the same
Farm core source and toolchain. The DLL owns a write guard on the host's
`CompilationContext.watch_graph` or a mutex guard on `resources_map`; the host
reader must wake after that guard is released. A normal single-binary lock test
cannot cover this boundary.

Build the fixture with `npm run build`. Set `CARGO_TARGET_DIR` to the root
repository's `target` directory and `FARM_SHARED_LOCK_LIBRARY` to the produced
library, then run `cargo test --release --target <host-triple> -p
farm-test-shared-lock-host`. On Windows x64 the library is
`target/x86_64-pc-windows-msvc/release/farm_test_shared_lock_plugin.dll`.

The pointers are exclusively test-owned. On failure the parked reader keeps
its Arc alive until the test process exits; the test does not destroy a lock
that still has a waiter. This is not a published plugin or a performance test.
