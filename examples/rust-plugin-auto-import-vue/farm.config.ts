import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
import farmAutoImport from '@farmfe/plugin-auto-import';
import visualizer from '@farmfe/js-plugin-visualizer';
export default defineConfig({
  vitePlugins: [vue()],
  plugins: [
    visualizer(),
    farmAutoImport({
      dts: "./src/auto_import.d.ts",
      presets:[
        "vue",
        {
          '@vueuse/core': [
            'useMouse',
            ['useFetch', 'useMyFetch']
          ],
        }
      ]
    })],
});
