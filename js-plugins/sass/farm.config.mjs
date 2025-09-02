import farmDtsPlugin from '@farmfe/js-plugin-dts';
import { createFarmJsPluginBuildConfig } from '../../configs/farm-js-plugin.base.config.mjs';

/**
 * @type {import('farm').UserConfig}
 */
export default createFarmJsPluginBuildConfig([
  farmDtsPlugin({
    tsConfigPath: './tsconfig.build.json'
  })
]);
