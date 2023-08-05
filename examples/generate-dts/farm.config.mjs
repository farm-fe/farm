import farmDtsPlugin from '@farmfe/js-plugin-dts';
import { builtinModules } from 'module';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: './index.ts'
    },
    sourcemap: false
  },
  plugins: [farmDtsPlugin()]
};
