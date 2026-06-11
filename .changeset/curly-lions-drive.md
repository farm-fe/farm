---
"@farmfe/core": patch
---

Fix SWC plugin transforms hanging during example builds on macOS and Windows.

Plugin transforms now run on a dedicated, process-wide multi-threaded tokio
runtime that is shared across all Farm worker threads, instead of constructing
a fresh runtime per call or holding a global serialization lock. The shared
`PluginRuntime` and compiled-plugin bytecode cache continue to be reused
across modules and threads, while module-level and plugin-level concurrency
are no longer artificially serialized.
