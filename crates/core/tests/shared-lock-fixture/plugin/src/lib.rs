use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use farmfe_core::context::CompilationContext;

/// Test-only ABI: all pointers are owned by the host and remain alive until
/// this synchronous call and its concurrent reader have finished.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hold_watch_graph(
  context: *const CompilationContext,
  writer_ready: *const AtomicBool,
  reader_started: *const AtomicBool,
) {
  // SAFETY: the host keeps its Arc and both atomics alive through the call.
  let (context, writer_ready, reader_started) =
    unsafe { (&*context, &*writer_ready, &*reader_started) };
  let mut graph = context.watch_graph.write();
  graph.add_node("cross-library-module".into());
  writer_ready.store(true, Ordering::Release);
  while !reader_started.load(Ordering::Acquire) {
    std::thread::yield_now();
  }
  // Give the host reader time to park while the DLL owns the write guard.
  std::thread::sleep(Duration::from_millis(300));
}

/// Same test-only pointer lifetime contract as hold_watch_graph.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hold_resources(
  context: *const CompilationContext,
  writer_ready: *const AtomicBool,
  reader_started: *const AtomicBool,
) {
  // SAFETY: the host retains all owners throughout this synchronous call.
  let (context, writer_ready, reader_started) =
    unsafe { (&*context, &*writer_ready, &*reader_started) };
  let _resources = context.resources_map.lock();
  writer_ready.store(true, Ordering::Release);
  while !reader_started.load(Ordering::Acquire) {
    std::thread::yield_now();
  }
  std::thread::sleep(Duration::from_millis(300));
}
