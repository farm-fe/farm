---
"@farmfe/plugin-dts": patch
"@farmfe/core": patch
---

Fix CSS url() resolution issues: support empty string URLs and correctly handle modules with multiple resources via new is_primary field in EmitFileParams
