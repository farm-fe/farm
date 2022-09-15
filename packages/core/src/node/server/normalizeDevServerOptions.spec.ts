import { test, expect } from 'vitest';
import {
  normalizeDevServerOptions,
  DEFAULT_DEV_SERVER_OPTIONS,
} from './normalizeDevServerOptions';

test('normalize-dev-server-options', () => {
  let options = normalizeDevServerOptions({});
  expect(options.https).toBe(DEFAULT_DEV_SERVER_OPTIONS.https);
  expect(options.port).toBe(DEFAULT_DEV_SERVER_OPTIONS.port);
  expect(options.writeToDisk).toBe(DEFAULT_DEV_SERVER_OPTIONS.writeToDisk);

  options = normalizeDevServerOptions({ writeToDisk: true });
  expect(options.https).toBe(DEFAULT_DEV_SERVER_OPTIONS.https);
  expect(options.port).toBe(DEFAULT_DEV_SERVER_OPTIONS.port);
  expect(options.writeToDisk).toBe(true);
});
