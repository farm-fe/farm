import { defineConfig } from 'farm';
import electron from '@farmfe/js-plugin-electron';

export default defineConfig({
  plugins: [
    '@farmfe/plugin-react',
    electron({
      main: {
        input: 'electron/main.ts',
      },
      preload: {
        input: 'electron/preload.ts',
      },
    }),
  ]
});
