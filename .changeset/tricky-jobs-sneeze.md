---
"@farmfe/core": patch
"@farmfe/cli": patch
---

- Mark farm compatible with node 16
- Support targetEnv `library-node` and `library-browser`
- fix watcher does not watch file change beyond the project root
- remove script bundle port conflict log when lazy compile is disabled