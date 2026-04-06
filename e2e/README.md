# E2E Guide

This folder contains Playwright-based E2E helpers used by example tests in `examples/*/e2e.spec.ts`.

## Add or Update an Example E2E Case

1. Use the skill `/e2e-example-acceptance` before and after edits.
2. Create or update `examples/<name>/e2e.spec.ts`.
3. Reuse `startProjectAndTest` from `e2e/vitestSetup.ts`.
4. Validate all three modes for the affected example:
   - `cd examples/<name> && npm run start`
   - `cd examples/<name> && npm run build`
   - `cd examples/<name> && npm run preview`
5. Ensure assertions cover:
   - expected page content
   - no runtime console errors
   - no unexpected failed requests

## Run a Single Example E2E

```bash
pnpm vitest run examples/<name>/e2e.spec.ts
```

## Run the E2E Suite

```bash
pnpm run test-e2e
```

## Test Design Tips

- Keep selectors stable (`data-testid` preferred for dynamic output).
- Prefer deterministic assertions over long fixed delays.
- Test both `start` and `preview` unless the example is intentionally single-mode.
