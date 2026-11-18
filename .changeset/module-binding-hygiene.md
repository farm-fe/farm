---
"@farmfe/core": patch
---

A top-level `module`, `exports`, `farmRequire` or `farmDynamicRequire` binding in an ES module is renamed again, so it no longer collides with the parameters of the module wrapper (`SyntaxError: Identifier 'module' has already been declared`).
