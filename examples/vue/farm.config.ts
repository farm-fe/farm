import { defineConfig } from 'farm';
import farmJsPluginVue from '@farmfe/js-plugin-vue';
import sass from '@farmfe/js-plugin-sass';

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    },
    persistentCache: false
  },
  plugins: [
    farmJsPluginVue(),
    sass({ legacy: true }),
    {
      name: 'remove-css-filter-plugin',
      priority: 0,
      transform: {
        filters: {
          resolvedPaths: ['.scss']
        },
        async executor({ content }) {
          return {
            content: content.replaceAll('filter: alpha(opacity=0);', '')
          };
        }
      }
    }
  ]
});
