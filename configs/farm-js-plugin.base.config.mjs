import { builtinModules } from 'module';

const format = process.env.FARM_FORMAT || 'cjs';
const ext = format === 'esm' ? 'mjs' : 'cjs';

export function createFarmJsPluginBuildConfig(plugins, options = {}) {
  return {
    compilation: {
      input: {
        index: './src/index.ts'
      },
      output: {
        path: `build/${format}`,
        entryFilename: `[entryName].${ext}`,
        targetEnv: 'node',
        format
      },
      partialBundling: {
        enforceResources: [
          {
            name: 'index.js',
            test: ['.+']
          }
        ]
      },
      minify: false,
      sourcemap: false,
      presetEnv: false,
      persistentCache: {
        envs: {
          FARM_FORMAT: format
        }
      }
    },
    server: {
      hmr: false
    },
    plugins
  };
}
