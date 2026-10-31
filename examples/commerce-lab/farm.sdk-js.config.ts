import dts from '@farmfe/js-plugin-dts';
import path from 'node:path';
import { sdkCommon } from './configs/sdk-common';
const common = sdkCommon('js');
export default {
  ...common,
  plugins: [dts({ outputDir: common.compilation.output.path, entryRoot: path.resolve('packages/sdk'), include: ['src/**/*.ts'], skipDiagnostics: false })],
};
