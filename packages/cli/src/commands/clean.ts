import { defineCommand } from 'citty';
import { FarmCLICommonOptions, ICleanOptions } from '../types.js';
import { resolveCliConfig, resolveCore } from '../utils.js';

export default defineCommand({
  meta: {
    name: 'clean',
    description: 'Clean up the cache built incrementally'
  },
  args: {
    root: {
      type: 'positional',
      description: 'root path',
      required: false,
      valueHint: 'path'
    },
    recursive: {
      type: 'boolean',
      alias: 'r',
      description:
        'Recursively search for node_modules directories and clean them'
    }
  },
  async run({ args }: { args: FarmCLICommonOptions & ICleanOptions }) {
    const { root } = resolveCliConfig(args.root, args.config);

    const { clean } = await resolveCore();
    try {
      await clean(root, args.recursive);
    } catch (e) {
      const { Logger } = await import('@farmfe/core');
      const logger = new Logger();
      logger.error(`Failed to clean cache: \n ${e.stack}`);
      process.exit(1);
    }
  }
});
