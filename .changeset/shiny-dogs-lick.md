---
"@farmfe/core": patch
---

- Fix dynamic import not work in webview context like vscode extension, we have specify the full lazy compilation path like `http://127.0.0.1:9000/__lazyCompile` instead of `/__layzeCompile`
- Support `compilation.output.clean` to control remove `output.path` or not, default to true
