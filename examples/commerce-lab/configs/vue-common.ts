import { common } from './common';

export function vueCommon(group: 'js' | 'rust') {
  const config = common(group);
  config.compilation!.input = { index: './apps/vue-lab/index.html' };
  config.compilation!.output = { path: `dist/vue-${group}` };
  config.compilation!.persistentCache = { cacheDir: `.cache/vue-${group}/${process.env.COMMERCE_CACHE_ID ?? 'default'}` };
  config.compilation!.define = {
    __VUE_OPTIONS_API__: 'true',
    __VUE_PROD_DEVTOOLS__: 'false',
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: 'false',
  };
  return config;
}
