import type { UserConfig } from '@farmfe/core';
import less from '@farmfe/js-plugin-less';

export default <UserConfig>{
  compilation: {
    resolve: {
      alias: {
        'stream$': 'readable-stream'
      }
    }
  },
  plugins: [
    less({}),
  ]
};
