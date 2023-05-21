---
'@farmfe/core': patch
---

- Add missing dependencies execa
- Add ./ to config.input when the values of config.input is not absolute path and do not start with ./
- Alias resolve take precedent over all other resolve strategies
- Do not resolve html dependencies starts with `http` and `/`
