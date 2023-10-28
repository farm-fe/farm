---
'@farmfe/plugin-sass': patch
'@farmfe/core': patch
---

Fix bugs:
1. `server.proxy` does not work as expected
2. `plugin-css` should treat `xxx.png` as relative path
3. `assets` like `/logo.png` under publicDir should be resolved to `publicDir/logo.png`