import type { UserConfig } from '@farmfe/core';
// import vue from '@farmfe/js-plugin-vue';
import vue from '@vitejs/plugin-vue';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  // plugins: [vue(), {
  //   name: 'test-plugin',
  //   load: {
  //     filters: { resolvedPaths: ['.*'] },
  //     executor(params, context) {
  //       console.log(params.resolvedPath);
  //       console.log(context?.viteGetModulesByFile(params.resolvedPath));
  //     }
  //   }
  // }],
  vitePlugins: [vue()]
});
