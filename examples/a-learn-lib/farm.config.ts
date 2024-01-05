import { builtinModules } from 'module';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    persistentCache: false,
    input: {
      index: './index.ts'
    },
    output: {
      path: 'dist',
      format: 'cjs',
      targetEnv: 'node'
    },
    partialBundling: {
      enforceResources: [
        {
          name: 'node.bundle.js',
          test: ['.+']
        }
      ]
    }
  },
  plugins: [
    {
      name: 'farm-plugin-learn-lib',
      finish: {
        executor() {
          console.log('finish');
        }
      },
      buildStart: {
        executor() {
          console.log('buildStart');
        }
      },
      buildEnd: {
        executor() {
          console.log('buildEnd');
        }
      },
      writeResources: {
        async executor(params) {
          console.log('writeResources');
        }
      },
      finalizeResources: {
        executor(params) {
          console.log('finalizeResources');
        }
      },
      renderStart: {
        executor() {
          console.log('renderStart');
        }
      },
      augmentResourceHash: {
        executor() {
          console.log('augmentResourceHash');
        }
      },
      renderResourcePot: {
        executor() {
          console.log('renderResourcePot');
        }
      }
    }
  ]
};
