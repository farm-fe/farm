---
"@farmfe/core": patch
---

Tree shaking no longer removes `x++` / `--x` updates and `delete` statements on top-level variables that are read later.
