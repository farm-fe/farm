import { defineConfig } from 'farm';
import farmPostcssPlugin from '@farmfe/js-plugin-postcss';
import path from 'path';


export default defineConfig({
  plugins: [farmPostcssPlugin({
    internalPlugins: {
      postcssImport: true
    }
  })],
  compilation: {
    resolve: {
      alias: {
        "@/": path.resolve(process.cwd(), 'src')
      }
    },
    persistentCache:false
  },
  publicDir: "public"
});
