import { defineConfig } from '@farmfe/core';
import visualizer from '@farmfe/js-plugin-visualizer';
import farmAutoImport from '@farmfe/plugin-auto-import';
import vue from '@vitejs/plugin-vue';
export default defineConfig({
  compilation: {
    sourcemap: false,
    persistentCache: true,
    minify: {
      mangleExports: true
    },
    concatenateModules: true
  },
  vitePlugins: [vue()],
  plugins: [
    process.env.FARM_VISUALIZER ? visualizer() : null,
    farmAutoImport({
      dts: './src/auto_import.d.ts',
      presets: [
        'vue',
        {
          '@vueuse/core': ['useMouse', ['useFetch', 'useMyFetch']]
        }
      ]
    })
  ].filter(Boolean)
});
