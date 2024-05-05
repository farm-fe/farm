import farmDtsPlugin from '@farmfe/js-plugin-dts';
import { createFarmJsPluginBuildConfig } from '../../configs/farm-js-plugin.base.config.mjs';

export default createFarmJsPluginBuildConfig([
  farmDtsPlugin({
    tsConfigPath: './tsconfig.build.json'
  })
]);
