import { Config } from '../../../binding/index.js';

export async function normalizeOutput(
  config: Config['config'],
  isProduction: boolean
) {
  if (isProduction) {
    if (!config.output) {
      config.output = {};
    }
    if (!config.output.filename) {
      config.output.filename = '[resourceName].[contentHash].[ext]';
    }
    if (!config.output.assetsFilename) {
      config.output.assetsFilename = '[resourceName].[contentHash].[ext]';
    }
  }

  normalizeTargetEnv(config);
}

type TargetEnvKeys = Config['config']['output']['targetEnv'];

type TargetsForTargetEnv = Record<
  TargetEnvKeys,
  {
    scriptTargets: string[] | null;
    cssTargets: string[] | null;
  } | null
>;
type TargetsMap = Omit<TargetsForTargetEnv, 'node' | 'browser'>;
const targetsMap: TargetsMap = {
  node16: {
    scriptTargets: ['node 16'],
    cssTargets: null
  },
  'node-legacy': {
    scriptTargets: ['node 10'],
    cssTargets: null
  },
  'node-next': null,
  'browser-legacy': {
    scriptTargets: ['> 0.25%, not dead'],
    cssTargets: ['> 0.25%, not dead']
  },
  'browser-es2015': {
    scriptTargets: ['fully supports es6'],
    cssTargets: ['fully supports es6']
  },
  'browser-es2017': {
    scriptTargets: ['fully supports async-functions'],
    cssTargets: ['fully supports async-functions']
  },
  'browser-esnext': null
};

/**
 * Set up targets for the given targetEnv.
 * @param config
 * @param isProduction
 */
function normalizeTargetEnv(config: Config['config']) {
  if (!config.output.targetEnv) {
    config.output.targetEnv = 'browser';
  }

  const aliasMap: Record<string, keyof TargetsMap> = {
    node: 'node16',
    browser: 'browser-es2017'
  };

  const targetEnv = (aliasMap[config.output.targetEnv] ??
    config.output.targetEnv) as keyof TargetsMap;

  if (targetsMap[targetEnv]) {
    const { scriptTargets, cssTargets } = targetsMap[targetEnv];
    // set defaults for targets
    if (config.presetEnv !== false) {
      // null means disable presetEnv
      if (scriptTargets == null) {
        config.presetEnv = false;
      } else if (typeof config.presetEnv === 'object') {
        if (!config.presetEnv.targets) {
          config.presetEnv.targets = scriptTargets;
        }
      } else {
        config.presetEnv = {
          targets: scriptTargets
        };
      }
    }

    if (config.css?.prefixer !== null) {
      if (cssTargets == null) {
        config.css ??= {};
        config.css.prefixer = null;
      } else if (typeof config.css?.prefixer === 'object') {
        if (!config.css.prefixer.targets) {
          config.css.prefixer.targets = cssTargets;
        }
      } else {
        config.css ??= {};
        config.css.prefixer = {
          targets: cssTargets
        };
      }
    }
  } else {
    // disable presetEnv and prefixer
    config.presetEnv = false;
    config.css.prefixer = null;
  }

  // the rust compiler only receives 'node' or 'browser'.
  if (
    ['node16', 'node-legacy', 'node-next'].includes(config.output.targetEnv)
  ) {
    config.output.targetEnv = 'node';
  } else if (
    [
      'browser-legacy',
      'browser-es2015',
      'browser-es2017',
      'browser-esnext'
    ].includes(config.output.targetEnv)
  ) {
    config.output.targetEnv = 'browser';
  }
}
