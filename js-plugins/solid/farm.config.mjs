import farmDtsPlugin from '@farmfe/js-plugin-dts';
import { createFarmJsPluginBuildConfig } from '../../configs/farm-js-plugin.base.config.mjs';

export default createFarmJsPluginBuildConfig([farmDtsPlugin()], {
  external: [
    '@babel/core',
    '@babel/preset-typescript',
    '@rollup/pluginutils',
    'babel-preset-solid',
    'merge-anything',
    'solid-refresh'
  ]
});
