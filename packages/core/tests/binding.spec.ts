import path from 'path';
import { fileURLToPath } from 'url';
import { test } from 'vitest';
import {
  Compiler,
  UserConfig,
  normalizeDevServerConfig,
  normalizeUserCompilationConfig,
  resolveUserConfig
} from '../src/index.js';

// just make sure the binding works
test('Binding - should parse config to rust correctly', async () => {
  const currentDir = path.dirname(fileURLToPath(import.meta.url));
  const serverConfig = normalizeDevServerConfig({});

  const config: UserConfig = {
    root: path.resolve(currentDir, 'fixtures', 'binding'),
    compilation: {
      progress: false
    },
    server: serverConfig
  };
  const resolvedUserConfig = await resolveUserConfig(config, undefined);
  const compilationConfig =
    await normalizeUserCompilationConfig(resolvedUserConfig);
  compilationConfig.persistentCache = false;
  console.log(compilationConfig);
  const compiler = new Compiler({
    compilation: compilationConfig,
    jsPlugins: [],
    rustPlugins: []
  });
  await compiler.compile();
});
