---
'@farmfe/cli': patch
---

Remove the unused `inquirer` dependency from `@farmfe/cli`. It pulled in `external-editor` -> `tmp`, which surfaced GHSA-52f5-9888-hmc6 (high) and GHSA-ph9p-34f9-6g65 on `npm i @farmfe/cli`. Nothing in the CLI imported `inquirer`, so installing the CLI now reports no advisories.
