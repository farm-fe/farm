import { builtinModules } from 'module';

/**
 * @type {import('farm').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: './index.ts'
    },
    output: {
      path: 'dist',
      targetEnv: 'node',
      entryFilename: '[entryName].mjs'
    },
    external: [
      ...builtinModules.map((m) => `^node:${m}$`),
      ...builtinModules.map((m) => `^${m}$`)
    ],
    minify: false,
    presetEnv: false
  },
  server: {
    hmr: false
  },
  plugins: [pluginCache()]
};

function pluginCache() {
  let globalCache = null;

  return {
    name: 'plugin-cache-example',
    pluginCacheLoaded: {
      executor(cache) {
        globalCache = Buffer.from(cache).toString('utf8');
      }
    },
    transform: {
      filters: {
        resolvedPaths: ['js-plugin-cache/index.ts']
      },
      executor({ content }) {
        globalCache = content;
        return { content };
      }
    },
    writePluginCache: {
      executor() {
        if (globalCache) {
          return [...Buffer.from(globalCache)];
        }
      }
    },
    finish: {
      executor() {
        console.log('globalCache', globalCache);
      }
    }
  };
}
