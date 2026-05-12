use std::collections::HashSet;

/// Registry of built-in variants.
pub struct VariantRegistry {
  variants: HashSet<&'static str>,
}

impl VariantRegistry {
  /// Create the built-in variant registry.
  pub fn builtin() -> Self {
    let variants: HashSet<&'static str> = [
      // Pseudo-classes
      "hover",
      "focus",
      "active",
      "disabled",
      "enabled",
      "visited",
      "target",
      "first",
      "last",
      "only",
      "odd",
      "even",
      "first-of-type",
      "last-of-type",
      "only-of-type",
      "empty",
      "required",
      "valid",
      "invalid",
      "checked",
      "indeterminate",
      "default",
      "optional",
      "in-range",
      "out-of-range",
      "read-only",
      "read-write",
      "placeholder-shown",
      "autofill",
      "focus-within",
      "focus-visible",
      "open",
      "inert",
      // Pseudo-elements
      "before",
      "after",
      "first-letter",
      "first-line",
      "marker",
      "selection",
      "file",
      "placeholder",
      "backdrop",
      // Directional
      "ltr",
      "rtl",
      // Media / responsive
      "dark",
      "print",
      "motion-safe",
      "motion-reduce",
      "portrait",
      "landscape",
      "contrast-more",
      "contrast-less",
      "starting",
      // Breakpoints (Tailwind v4 defaults)
      "sm",
      "md",
      "lg",
      "xl",
      "2xl",
      // Container queries
      "@sm",
      "@md",
      "@lg",
      "@xl",
      // Group / peer
      "group",
      "peer",
      "group-hover",
      "group-focus",
      "peer-hover",
      "peer-focus",
    ]
    .into_iter()
    .collect();

    Self { variants }
  }

  /// Returns `true` if the variant name is registered.
  pub fn has(&self, name: &str) -> bool {
    self.variants.contains(name)
  }
}
