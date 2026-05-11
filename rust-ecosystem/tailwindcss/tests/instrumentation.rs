mod support;

#[allow(dead_code)]
mod instrumentation {
  include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/generated/instrumentation.rs"
  ));

  #[cfg(test)]
  mod moved_tests {
    use super::*;
    use farmfe_testing_helpers::assert_snapshot;
    use regex::Regex;

    fn capture_report(i: &mut Instrumentation) -> String {
      let mut buf = String::new();
      i.report(|msg| buf.push_str(msg));
      Regex::new(r"\[\s*\d+\.\d+ms\]")
        .unwrap()
        .replace_all(&strip_ansi(&buf), "[TIME]")
        .into_owned()
    }

    #[test]
    fn report_snapshots_cover_hits_and_timers() {
      let mut hits = Instrumentation::new();
      hits.hit("Potato");
      hits.hit("Potato");
      hits.hit("Potato");

      let mut nested = Instrumentation::new();
      nested.start("Foo");
      nested.start("Bar");
      nested.end("Bar");
      nested.end("Foo");

      let mut repeated = Instrumentation::new();
      repeated.start("Foo");
      for _ in 0..5 {
        repeated.start("Inner");
        repeated.end("Inner");
      }
      repeated.end("Foo");

      let mut mixed = Instrumentation::new();
      mixed.start("Resolve");
      mixed.end("Resolve");
      mixed.hit("cache_miss");
      mixed.hit("cache_miss");

      let snapshot = format!(
        "hits:\n{}\nnested:\n{}\nrepeated:\n{}\nmixed:\n{}",
        capture_report(&mut hits),
        capture_report(&mut nested),
        capture_report(&mut repeated),
        capture_report(&mut mixed),
      );

      assert_snapshot!(snapshot);
    }

    #[test]
    fn reset_clears_data() {
      let mut instrumentation = Instrumentation::new();
      instrumentation.hit("Foo");
      instrumentation.start("Bar");
      instrumentation.end("Bar");
      instrumentation.reset();
      assert!(capture_report(&mut instrumentation).trim().is_empty());
    }

    #[test]
    #[should_panic(expected = "Mismatched timer label")]
    fn mismatched_end_panics() {
      let mut instrumentation = Instrumentation::new();
      instrumentation.start("Foo");
      instrumentation.end("NotFoo");
    }

    #[test]
    fn strip_ansi_helper_stays_stable() {
      let with_codes = "\x1b[2mhello\x1b[22m \x1b[34mworld\x1b[39m";
      assert_eq!(strip_ansi(with_codes), "hello world");
      assert_eq!(strip_ansi("hello world"), "hello world");
    }
  }
}
