import path from 'node:path';
import { JsPlugin, UserConfig } from '../../src/index.js';
import {
  getFixturesDir,
  getCompiler as getInternalCompiler
} from '../common.js';

export function getJsPluginsFixturesDir(hookName: string) {
  return path.resolve(getFixturesDir(), 'js-plugins-hooks', hookName);
}

export function getOutputFilePath(p: string, hookName: string) {
  const root = getJsPluginsFixturesDir(hookName);
  return path.join(root, 'dist', p, 'index.mjs');
}

export function getCompiler(
  p: string,
  plugins: JsPlugin[],
  hookName: string,
  input?: Record<string, string>,
  output?: Record<string, string>,
  compilation?: UserConfig['compilation']
) {
  const root = getJsPluginsFixturesDir(hookName);
  return getInternalCompiler(root, p, plugins, input, output, compilation);
}
