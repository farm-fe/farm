---
"@farmfe/plugin-worker": patch
---

Fix inline worker resource patching when generated JavaScript resources contain non-UTF8 bytes, preventing worker placeholders from being erased before replacement.
