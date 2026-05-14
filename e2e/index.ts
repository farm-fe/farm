/**
 * Public API surface for Farm's standalone E2E test infrastructure.
 *
 * Spec files should import from here:
 *
 *   import { startAndTest, watchAndTest, expect } from '../../e2e/index.ts';
 *   import type { SpecContext } from '../../e2e/index.ts';
 */
export { startAndTest, watchAndTest, initBrowser, initBrowserContext } from './farm-runner.ts';
export type { Page } from './farm-runner.ts';

export { expect } from './expect.ts';

export { SpecRunner, printSummary } from './runner.ts';
export type { SpecContext, SpecFn, TestResult } from './runner.ts';

export { editFile, logger } from './utils.ts';

/** SSR examples that require a different test strategy. */
export const ssrExamples: string[] = ['react-ssr', 'vue-ssr', 'solid-ssr'];
