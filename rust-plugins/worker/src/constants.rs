/// Matches `?worker`, `?sharedworker`, `&worker`, `&sharedworker` query parameters.
/// Capture group 1 is the constructor name (`worker` or `sharedworker`).
pub const WORKER_OR_SHARED_WORKER_RE: &str = r#"(?:\?|&)(worker|sharedworker)(?:&|$)"#;

/// Matches `new Worker(new URL("...", import.meta.url))` and
/// `new SharedWorker(new URL("...", import.meta.url))` call expressions.
/// Capture group 0 is the full argument list; capture group 1 is the `new URL(...)` part.
pub const WORKER_IMPORT_META_URL_RE: &str = r#"\bnew\s+(?:Worker|SharedWorker)\s*\(\s*(new\s+URL\s*\(\s*('[^']+'|"[^"]+"|`[^`]+`)\s*,\s*import\.meta\.url[^)]*\))"#;
