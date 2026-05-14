/**
 * LEGACY: This file is no longer used.
 *
 * The new standalone E2E runner (`scripts/test-e2e.mts`) manages the
 * browser lifecycle directly without vitest.  It launches a Playwright
 * Chromium browser once before running all specs, then closes it at the end.
 *
 * See:
 *   - scripts/test-e2e.mts (new runner)
 *   - e2e/farm-runner.ts (browser + process utilities)
 *   - e2e/runner.ts (test framework)
 */

