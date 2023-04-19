// change to @farmfe/core/config when resolve support conditional exports
import { defineFarmConfig } from '@farmfe/core/config';

export default defineFarmConfig({
  plugins: [['@farmfe/plugin-vue-jsx', { name: 'erkelost' }]],
});
