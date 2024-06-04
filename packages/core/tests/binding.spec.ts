import path from 'path';
import { fileURLToPath } from 'url';
import { test } from 'vitest';
import {
  Compiler,
  Logger,
  UserConfig,
  normalizeDevServerOptions,
  normalizeUserCompilationConfig,
  resolveMergedUserConfig
} from '../src/index.js';

// just make sure the binding works
test('Binding - should parse config to rust correctly', async () => {
  const currentDir = path.dirname(fileURLToPath(import.meta.url));
  const serverConfig = normalizeDevServerOptions({}, 'production');

  const config: UserConfig = {
    root: path.resolve(currentDir, 'fixtures', 'binding'),
    compilation: {
      progress: false
    },
    server: serverConfig
  };
  const resolvedUserConfig = await resolveMergedUserConfig(
    config,
    undefined,
    'production'
  );
  const compilationConfig = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    config,
    new Logger()
  );
  const compiler = new Compiler({
    config: compilationConfig,
    jsPlugins: [],
    rustPlugins: []
  });
  await compiler.compile();
});
