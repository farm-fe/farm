---
"@farmfe/core": patch
---

Fix SWC plugin transforms hanging during example builds on macOS and Windows
CI runners (reproducible at `FARM_THREAD_NUMS=2`).

The hang was traced to wasmer's `Engine::compile` / `Engine::validate` paths
contending on an internal `std::sync::Mutex` when multiple Farm rayon worker
threads ran wasm plugin transforms in parallel. We now serialize the
`transform_plugin_executor.transform()` call via a process-wide mutex while
keeping every other build phase fully parallel.

In addition, the plugin host's `block_on` now always runs on a dedicated,
process-wide multi-threaded tokio runtime (constructed once instead of
per-call) and uses `tokio::task::block_in_place` when invoked from inside an
ambient tokio runtime, avoiding worker-thread starvation.
