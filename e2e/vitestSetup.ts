/**
 * LEGACY: This file is no longer used.
 *
 * The old E2E infrastructure was built on top of vitest, using:
 *   - vitestGlobalSetup.ts to launch a shared browser
 *   - vitestSetup.ts to provide test utilities like startProjectAndTest()
 *   - vitest.config.e2e.ts to configure the runner
 *
 * The new standalone E2E runner completely replaces this, providing:
 *   - e2e/farm-runner.ts - startAndTest(), watchAndTest() (subprocess + playwright)
 *   - e2e/runner.ts - SpecRunner, SpecContext (async test framework)
 *   - e2e/expect.ts - minimal expect() implementation
 *   - scripts/test-e2e.mts - browser lifecycle + orchestration
 *
 * No vitest dependency at all.  Spec files are TypeScript modules that
 * export a default async function.
 */
