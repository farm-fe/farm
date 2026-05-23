# E2E Guide

This folder contains a standalone Playwright-based E2E test runner (no vitest). See `scripts/test-e2e.mts` for the main orchestrator.

## Add or Update an Example E2E Case

1. Use the skill `/e2e-example-acceptance` before and after edits.
2. Create or update `examples/<name>/e2e.spec.ts`.
3. Reuse `startAndTest` and `watchAndTest` from `e2e/farm-runner.ts`. Import the `expect` function from `e2e/expect.ts`.
4. Validate all three modes for the affected example:
   - `cd examples/<name> && npm run start`
   - `cd examples/<name> && npm run build`
   - `cd examples/<name> && npm run preview`
5. Ensure assertions cover:
   - expected page content
   - no runtime console errors
   - no unexpected failed requests

## Run a Single Example E2E

Use the new standalone runner with example filtering:

```bash
pnpm run test-e2e -- --example <name>
# Or use project alias:
pnpm run test-e2e -- --project <name>
```

## Run from a Specific Example Onward

```bash
pnpm run test-e2e -- --from <name>
# Also works with --start-from <name>
```

## Run the E2E Suite

```bash
pnpm run test-e2e
```

## Test Design Tips

- Spec files export a default async function that receives a SpecContext (`ctx.test()`, `ctx.describe()`).
- Keep selectors stable (`data-testid` preferred for dynamic output).
- Prefer deterministic assertions over long fixed delays.
- Test both `start` and `preview` unless the example is intentionally single-mode.

## Key Files

- `farm-runner.ts` - Playwright + subprocess utilities (`startAndTest`, `watchAndTest`, `visitPage`)
- `expect.ts` - Minimal jest-compatible expect() implementation
- `runner.ts` - Test framework (SpecRunner, SpecContext)
- `index.ts` - Public API exports
- `scripts/test-e2e.mts` - Main orchestrator (browser lifecycle, example discovery, result reporting)
