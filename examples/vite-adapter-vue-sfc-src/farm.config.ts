import { defineConfig, JsPlugin } from "farm";
import vue from '@vitejs/plugin-vue';
import path from 'path';
import { type Plugin } from 'vite';

const b: Plugin = {
  name: 'b',
  transform(code, id, options) {
    if (id.includes('a.ts')) {
      const dir = path.dirname(id);
      const resolvedPath = path.resolve(dir, 'style.css').replaceAll('\\', '\\\\');
      return {
        // code: `import './style.css?type=style&index=0&src=7a7a37b1&scoped=7a7a37b1&lang.css'`,
        code: `import '${resolvedPath}?vue&type=style&index=0&src=7a7a37b1&scoped=7a7a37b1&lang.css'`,
      };
    }
  },
};

export default defineConfig({
  vitePlugins: [vue(), b],
});
