---
"@farmfe/core": patch
---

Advance the internal native plugin ABI identifier after the shared resource and
source observation context layout changes. Rebuild Rust plugins against the same
V2 core before loading them; older native libraries are rejected by the loader.
