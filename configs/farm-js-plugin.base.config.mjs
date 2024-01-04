import { builtinModules } from 'module';

const format = process.env.FARM_FORMAT || 'cjs';
const ext = format === 'esm' ? 'mjs' : 'cjs';
console.log('format', format, ext);

export function createFarmJsPluginBuildConfig(plugins) {
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
      external: [
        ...builtinModules.map((m) => `^${m}$`),
        ...builtinModules.map((m) => `^node:${m}$`)
      ],
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
    plugins,
  };

}