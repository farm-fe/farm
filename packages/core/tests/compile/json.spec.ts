import path from 'path';
import { expect, test } from 'vitest';
import {
  getCompiler,
  getFixturesDir,
  getOutputFilePath,
  getOutputResult
} from '../common.js';

test('Json compilation', async () => {
  const root = path.join(getFixturesDir(), 'json');
  const compiler = await getCompiler(root, 'json', []);
  await compiler.compile();
  compiler.writeResourcesToDisk();
  const outputFilePath = getOutputFilePath(root, 'json');
  const result = await getOutputResult(outputFilePath);
  expect(result.default.json1Name).toBe('json1');
  expect(result.default.json2Name).toBe('json2');
});
