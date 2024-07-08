import { defineCommand } from 'citty';
import {
  FarmCLICommonOptions,
  FarmCLIPreviewOptions,
  GlobalFarmCLIOptions
} from '../types.js';
import {
  handleAsyncOperationErrors,
  resolveCliConfig,
  resolveCommandOptions,
  resolveCore
} from '../utils.js';

export default defineCommand({
  meta: {
    name: 'preview',
    description: 'compile the project in watch mode'
  },
  args: {
    root: {
      type: 'positional',
      description: 'root path',
      required: false,
      valueHint: 'path'
    },
    port: { type: 'string', description: 'specify port' },
    open: {
      type: 'boolean',
      description: 'open browser on server preview start'
    }
  },
  async run({ args }: { args: FarmCLICommonOptions & FarmCLIPreviewOptions }) {
    const { root, configPath } = resolveCliConfig(args.root, args.config);

    const resolvedOptions = resolveCommandOptions(args as GlobalFarmCLIOptions);
    const defaultOptions = {
      root,
      mode: args.mode,
      server: resolvedOptions,
      configPath,
      port: resolvedOptions.port
    };

    const { preview } = await resolveCore();
    handleAsyncOperationErrors(
      preview(defaultOptions),
      'Failed to start preview server'
    );
  }
});
