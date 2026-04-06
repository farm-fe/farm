---
name: e2e-example-acceptance
description: Validate or update example-level E2E coverage. Use when adding/updating e2e.spec.ts, debugging example start/build/preview behavior, or verifying browser console/request errors and page expectations.
license: MIT
compatibility: Requires Node, pnpm, and Playwright dependencies installed.
metadata:
  author: farm
  version: "1.0"
---

Run example acceptance with executable checks for start/build/preview and browser behavior.

## When to Use

- Adding a new `examples/<name>/e2e.spec.ts`
- Updating an existing E2E case for an example
- Verifying regression fixes in an example before merge
- Confirming no browser console/page/request anomalies during acceptance

## Mandatory Acceptance Flow

1. Identify affected examples

   - From changed files, collect all `examples/<name>/...` paths.
   - If no explicit list is provided, ask for the intended scope.

2. Build each affected example

   - `cd examples/<name> && npm run build`

3. Run start and preview checks for each affected example

   - For examples with dedicated `e2e.spec.ts`, run targeted E2E:
     - `pnpm vitest run examples/<name>/e2e.spec.ts`
   - For examples without a dedicated test, run smoke checks manually:
     - `cd examples/<name> && npm run start`
     - `cd examples/<name> && npm run preview`

4. Browser-level acceptance criteria

   - Console: no unhandled `error`/`pageerror` messages.
   - Network: no unexpected `requestfailed` entries.
   - Content: assert key selectors/text that prove feature correctness.

5. Update or add E2E tests when needed
   - Prefer dedicated `examples/<name>/e2e.spec.ts` for feature-specific behavior.
   - Keep assertions deterministic; avoid brittle timing-only checks.
   - Include both `start` and `preview` coverage unless the example is explicitly single-mode.

## Authoring Template

```ts
import { basename, dirname } from "path";
import { fileURLToPath } from "url";
import { expect, test } from "vitest";
import { startProjectAndTest } from "../../e2e/vitestSetup";

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

test(`e2e tests - ${name}`, async () => {
  const runTest = (command?: "start" | "preview") =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector("#root > *", { timeout: 10000 });
        // assert page content and absence of runtime anomalies
      },
      command,
    );

  await runTest();
  await runTest("preview");
});
```

## Reporting Format

- `Examples`: list of verified examples
- `Build`: pass/fail per example
- `E2E Start`: pass/fail per example
- `E2E Preview`: pass/fail per example
- `Anomalies`: console/request/pageerror summary
- `Notes`: expected skips or known caveats

## Guardrails

- Do not rely only on build success; start/preview behavior must be observed.
- Do not use `pnpm run ready` unless the user explicitly asks for full CI parity.
- Do not edit generated `dist/` files.
