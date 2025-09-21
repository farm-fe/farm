---
"@farmfe/js-plugin-visualizer": patch
"@farmfe/core": patch
---

Fix issues:

1. Move all custom config from `config.custom` to the corresponding config field. For example, move `config.custom.external` to `config.external`.
2. Fix visualizer plugin not working issue.
3. `?url` does not be removed in plugin static asset
