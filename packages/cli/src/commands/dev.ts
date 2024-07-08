import { defineCommand } from 'citty';
import {
  FarmCLICommonOptions,
  FarmCLIServerOptions,
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
    name: 'dev',
    description:
      'Compile the project in dev mode and serve it with farm dev server'
  },
  args: {
    root: {
      type: 'positional',
      description: 'root path',
      required: false,
      valueHint: 'path'
    },
    lazy: { type: 'boolean', alias: 'l', description: 'lazyCompilation' },
    host: { type: 'string', description: 'specify host' },
    port: { type: 'string', description: 'specify port' },
    open: { type: 'boolean', description: 'open browser on server start' },
    hmr: { type: 'boolean', description: 'enable hot module replacement' },
    cors: { type: 'boolean', description: 'enable cors' },
    strictPort: {
      type: 'boolean',
      description: 'specified port is already in use, exit with error'
    }
  },
  async run({ args }: { args: FarmCLICommonOptions & FarmCLIServerOptions }) {
    const { root, configPath } = resolveCliConfig(args.root, args.config);

    const resolvedOptions = resolveCommandOptions(args as GlobalFarmCLIOptions);
    const defaultOptions = {
      root,
      compilation: {
        lazyCompilation: args.lazy
      },
      server: resolvedOptions,
      clearScreen: args.clearScreen,
      configPath,
      mode: args.mode
    };
    const { start } = await resolveCore();
    handleAsyncOperationErrors(start(defaultOptions), 'Failed to start server');
  }
});
