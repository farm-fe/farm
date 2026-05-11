//! Performance instrumentation.
//!
//! Rust port of
//! [`@tailwindcss-node/src/instrumentation.ts`](https://github.com/tailwindlabs/tailwindcss/blob/main/packages/%40tailwindcss-node/src/instrumentation.ts).
//!
//! Provides an [`Instrumentation`] struct that records hit counts and
//! hierarchical elapsed-time measurements.  The output format matches the
//! upstream TypeScript version so that the same reporting conventions are
//! preserved.
//!
//! # Example
//!
//! ```
//! use farmfe_ecosystem_tailwindcss::instrumentation::Instrumentation;
//!
//! let mut i = Instrumentation::new();
//! i.start("resolve");
//! i.end("resolve");
//! i.hit("cache_hit");
//!
//! let mut output = String::new();
//! i.report(|msg| output.push_str(msg));
//! assert!(output.contains("cache_hit"));
//! assert!(output.contains("resolve"));
//! ```

use std::collections::HashMap;
use std::time::{Duration, Instant};

// ── internal types ────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
struct HitCounter {
    value: u64,
}

#[derive(Debug, Default, Clone)]
struct TimerAccum {
    value: Duration,
}

#[derive(Debug, Clone)]
struct TimerFrame {
    /// Fully-qualified ID, e.g. `"Foo//Bar"`
    id: String,
    /// Short label (the argument passed to `start`)
    label: String,
    /// The instant the timer was started
    started_at: Instant,
}

// ── public API ────────────────────────────────────────────────────────────────

/// A performance profiler that records hit counts and hierarchical timers.
///
/// Mirrors the `Instrumentation` class from `instrumentation.ts`.
pub struct Instrumentation {
    hits: HashMap<String, HitCounter>,
    timers: HashMap<String, TimerAccum>,
    /// Insertion-ordered timer IDs (to preserve report order)
    timer_order: Vec<String>,
    timer_stack: Vec<TimerFrame>,
    default_flush: Box<dyn Fn(&str) + Send>,
}

impl std::fmt::Debug for Instrumentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instrumentation")
            .field("hits", &self.hits)
            .field("timers", &self.timers)
            .field("timer_stack_depth", &self.timer_stack.len())
            .finish()
    }
}

impl Default for Instrumentation {
    fn default() -> Self {
        Self::new()
    }
}

impl Instrumentation {
    /// Create a new [`Instrumentation`] that writes reports to `stderr`.
    pub fn new() -> Self {
        Self::with_flush(|msg| eprint!("{msg}"))
    }

    /// Create a new [`Instrumentation`] with a custom flush function.
    pub fn with_flush(flush: impl Fn(&str) + Send + 'static) -> Self {
        Self {
            hits: HashMap::new(),
            timers: HashMap::new(),
            timer_order: Vec::new(),
            timer_stack: Vec::new(),
            default_flush: Box::new(flush),
        }
    }

    /// Record a single hit for `label`.
    ///
    /// Mirrors `I.hit(label)` in TypeScript.
    pub fn hit(&mut self, label: &str) {
        self.hits.entry(label.to_string()).or_default().value += 1;
    }

    /// Start a timer for `label`, recording the current instant.
    ///
    /// Mirrors `I.start(label)` in TypeScript.
    pub fn start(&mut self, label: &str) {
        // Build the fully qualified ID from the current stack
        let namespace: String = self
            .timer_stack
            .iter()
            .map(|f| f.label.as_str())
            .collect::<Vec<_>>()
            .join("//");

        let id = if namespace.is_empty() {
            label.to_string()
        } else {
            format!("{namespace}//{label}")
        };

        // Record a hit for this timer
        self.hits.entry(id.clone()).or_default().value += 1;

        // Ensure a timer accumulator exists (preserve order)
        if !self.timers.contains_key(&id) {
            self.timers.insert(id.clone(), TimerAccum::default());
            self.timer_order.push(id.clone());
        }

        self.timer_stack.push(TimerFrame {
            id,
            label: label.to_string(),
            started_at: Instant::now(),
        });
    }

    /// Stop the timer for `label` and accumulate elapsed time.
    ///
    /// # Panics
    /// Panics if `label` does not match the top of the timer stack (mirrors the
    /// upstream TypeScript which throws an `Error` in that case).
    ///
    /// Mirrors `I.end(label)` in TypeScript.
    pub fn end(&mut self, label: &str) {
        let elapsed = self
            .timer_stack
            .last()
            .map(|f| f.started_at.elapsed())
            .unwrap_or(Duration::ZERO);

        let frame = self.timer_stack.last().expect("end() called with empty stack");
        assert_eq!(
            frame.label, label,
            "Mismatched timer label: `{label}`, expected `{}`",
            frame.label
        );

        let id = frame.id.clone();
        self.timer_stack.pop();
        self.timers.entry(id).or_default().value += elapsed;
    }

    /// Reset all counters and timers.
    ///
    /// Mirrors `I.reset()` in TypeScript.
    pub fn reset(&mut self) {
        self.hits.clear();
        self.timers.clear();
        self.timer_order.clear();
        self.timer_stack.clear();
    }

    /// Auto-close pending timers and emit the report string.
    ///
    /// Accepts a callback that receives the formatted report string, matching
    /// the TypeScript `I.report((output) => { … })` API.
    ///
    /// Mirrors `I.report(flush?)` in TypeScript.
    pub fn report(&mut self, mut flush: impl FnMut(&str)) {
        // Auto-end any pending timers (innermost first)
        let labels: Vec<String> = self
            .timer_stack
            .iter()
            .rev()
            .map(|f| f.label.clone())
            .collect();
        for label in labels {
            self.end(&label);
        }

        let mut output: Vec<String> = Vec::new();
        let mut has_hits = false;

        // ── hit counters (non-timer entries) ─────────────────────────────────
        // Collect non-timer hits, sorted for deterministic output
        let mut hit_labels: Vec<(&String, u64)> = self
            .hits
            .iter()
            .filter(|(k, _)| !self.timers.contains_key(*k))
            .map(|(k, v)| (k, v.value))
            .collect();
        hit_labels.sort_by_key(|(k, _)| k.as_str());

        if !hit_labels.is_empty() {
            has_hits = true;
            output.push("Hits:".to_string());
        }

        for (label, count) in &hit_labels {
            let depth = label.split("//").count();
            output.push(format!(
                "{}{}{}",
                "  ".repeat(depth),
                label,
    dim(&blue(&format!(" × {count}")))
            ));
        }

        // ── timers ─────────────────────────────────────────────────────────────
        if !self.timers.is_empty() {
            if has_hits {
                output.push(String::new());
                output.push("Timers:".to_string());
            }

            // Compute maximum label width for alignment
            let max_width = self
                .timer_order
                .iter()
                .map(|id| {
                    let ms = self
                        .timers
                        .get(id)
                        .map(|t| t.value.as_secs_f64() * 1000.0)
                        .unwrap_or(0.0);
                    format!("{ms:.2}ms").len()
                })
                .max()
                .unwrap_or(0);

            for id in &self.timer_order {
                let depth = id.split("//").count();
                let ms = self
                    .timers
                    .get(id)
                    .map(|t| t.value.as_secs_f64() * 1000.0)
                    .unwrap_or(0.0);
                let ms_str = format!("{ms:.2}ms");
                let padded = format!("{ms_str:>max_width$}");
                let hit_count = self.hits.get(id).map(|h| h.value).unwrap_or(1);
                let short_label = id.split("//").last().unwrap_or(id);

                let hit_suffix = if hit_count == 1 {
                    String::new()
                } else {
                    format!(" {}", dim(&blue(&format!("× {hit_count}"))))
                };

                let indent = if depth == 1 {
                    " ".to_string()
                } else {
                    format!("{}{}", "  ".repeat(depth - 1), dim(" ↳ "))
                };

                output.push(
                    format!(
                        "{}{}{}{}",
                        dim(&format!("[{padded}]")),
                        indent,
                        short_label,
                        hit_suffix
                    )
                    .trim_end()
                    .to_string(),
                );
            }
        }

        flush(&format!("\n{}\n", output.join("\n")));
        self.reset();
    }

    /// Emit the report using the default flush function (stderr).
    pub fn report_default(&mut self) {
        // Temporarily capture output to avoid borrow issues
        let mut output = String::new();
        self.report(|msg| output.push_str(msg));
        (self.default_flush)(&output);
    }
}

// ── ANSI helpers ──────────────────────────────────────────────────────────────

fn dim(s: &str) -> String {
    format!("\x1b[2m{s}\x1b[22m")
}

fn blue(s: &str) -> String {
    format!("\x1b[34m{s}\x1b[39m")
}

/// Strip ANSI VT escape codes from a string (for test assertions).
pub fn strip_ansi(s: &str) -> String {
    // Remove ANSI escape sequences like \x1b[...m
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // consume until 'm'
            for ch in chars.by_ref() {
                if ch == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn capture_report(i: &mut Instrumentation) -> String {
        let mut buf = String::new();
        i.report(|msg| buf.push_str(msg));
        strip_ansi(&buf)
    }

    // ── hit counter ───────────────────────────────────────────────────────────

    #[test]
    fn hit_increments_counter() {
        let mut i = Instrumentation::new();
        i.hit("Potato");
        i.hit("Potato");
        i.hit("Potato");

        let report = capture_report(&mut i);
        assert!(report.contains("Hits:"), "report: {report}");
        assert!(report.contains("Potato"), "report: {report}");
        assert!(report.contains("× 3"), "report: {report}");
    }

    #[test]
    fn hit_different_labels_are_independent() {
        let mut i = Instrumentation::new();
        i.hit("Alpha");
        i.hit("Beta");
        i.hit("Beta");

        let report = capture_report(&mut i);
        assert!(report.contains("Alpha"), "report: {report}");
        assert!(report.contains("Beta"), "report: {report}");
        assert!(report.contains("× 2"), "report: {report}");
    }

    // ── timer ─────────────────────────────────────────────────────────────────

    #[test]
    fn start_end_records_timer() {
        let mut i = Instrumentation::new();
        i.start("Foo");
        i.end("Foo");

        let report = capture_report(&mut i);
        assert!(report.contains("Foo"), "report: {report}");
    }

    #[test]
    fn nested_timers_are_indented() {
        let mut i = Instrumentation::new();
        i.start("Foo");
        i.start("Bar");
        i.end("Bar");
        i.end("Foo");

        let report = capture_report(&mut i);
        assert!(report.contains("Foo"), "report: {report}");
        assert!(report.contains("Bar"), "report: {report}");
        // Bar should be indented (contains "↳")
        assert!(report.contains("↳"), "report: {report}");
    }

    #[test]
    fn timer_hit_count_shown_when_called_multiple_times() {
        let mut i = Instrumentation::new();
        i.start("Foo");
        for _ in 0..5 {
            i.start("Inner");
            i.end("Inner");
        }
        i.end("Foo");

        let report = capture_report(&mut i);
        // Inner was called 5 times, hit count should show × 5
        assert!(report.contains("× 5"), "report: {report}");
    }

    #[test]
    fn timer_called_once_no_hit_suffix() {
        let mut i = Instrumentation::new();
        i.start("OnceOnly");
        i.end("OnceOnly");

        let report = capture_report(&mut i);
        // When called once, no "× 1" should appear for that timer
        // (the upstream strips it too)
        assert!(!report.contains("× 1"), "report should not show × 1: {report}");
    }

    // ── auto-end pending timers ───────────────────────────────────────────────

    #[test]
    fn auto_ends_pending_timers() {
        let mut i = Instrumentation::new();
        i.start("Foo");
        for _ in 0..3 {
            i.start("Bar");
            i.end("Bar");
        }
        i.start("Baz"); // Baz not ended manually

        let report = capture_report(&mut i);
        assert!(report.contains("Foo"), "report: {report}");
        assert!(report.contains("Bar"), "report: {report}");
        assert!(report.contains("Baz"), "report: {report}");
    }

    // ── mixed hits + timers ───────────────────────────────────────────────────

    #[test]
    fn hits_and_timers_both_appear() {
        let mut i = Instrumentation::new();
        i.start("Resolve");
        i.end("Resolve");
        i.hit("cache_miss");
        i.hit("cache_miss");

        let report = capture_report(&mut i);
        assert!(report.contains("Hits:"), "report: {report}");
        assert!(report.contains("cache_miss"), "report: {report}");
        assert!(report.contains("Resolve"), "report: {report}");
    }

    // ── reset ─────────────────────────────────────────────────────────────────

    #[test]
    fn reset_clears_all_data() {
        let mut i = Instrumentation::new();
        i.hit("Foo");
        i.start("Bar");
        i.end("Bar");
        i.reset();

        let report = capture_report(&mut i);
        // After reset, report should be essentially empty (just newlines)
        let trimmed = report.trim();
        assert!(trimmed.is_empty(), "Expected empty report, got: {report}");
    }

    // ── mismatched end panics ─────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "Mismatched timer label")]
    fn mismatched_end_panics() {
        let mut i = Instrumentation::new();
        i.start("Foo");
        i.end("NotFoo"); // Should panic
    }

    // ── strip_ansi helper ─────────────────────────────────────────────────────

    #[test]
    fn strip_ansi_removes_escape_codes() {
        let with_codes = "\x1b[2mhello\x1b[22m \x1b[34mworld\x1b[39m";
        assert_eq!(strip_ansi(with_codes), "hello world");
    }

    #[test]
    fn strip_ansi_plain_string_unchanged() {
        let plain = "hello world";
        assert_eq!(strip_ansi(plain), "hello world");
    }

    // ── Instrumentation::default ──────────────────────────────────────────────

    #[test]
    fn default_instrumentation_is_usable() {
        let mut i = Instrumentation::default();
        i.hit("test");
        let report = capture_report(&mut i);
        assert!(report.contains("test"));
    }

    // ── deep nesting ──────────────────────────────────────────────────────────

    #[test]
    fn deeply_nested_timers() {
        let mut i = Instrumentation::new();
        i.start("A");
        i.start("B");
        i.start("C");
        i.end("C");
        i.end("B");
        i.end("A");

        let report = capture_report(&mut i);
        assert!(report.contains("A"), "report: {report}");
        assert!(report.contains("B"), "report: {report}");
        assert!(report.contains("C"), "report: {report}");
        // B and C should both be indented (↳)
        let arrow_count = report.matches('↳').count();
        assert_eq!(arrow_count, 2, "Expected 2 ↳ arrows, report: {report}");
    }
}
