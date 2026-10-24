import dts from '@farmfe/plugin-dts';
import { sdkCommon } from './configs/sdk-common';
export default { ...sdkCommon('rust'), plugins: [dts({ include: ['\\.ts$'], exclude: ['node_modules/'] })] };
