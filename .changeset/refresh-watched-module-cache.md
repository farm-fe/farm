---
"@farmfe/core": patch
---

Invalidate cached modules when watched dependencies change and rerun freeze hooks for expired modules even when their own source content is unchanged. Include watched content fingerprints in generated resource and development CSS caches so freeze-hook output is regenerated after candidate changes.
