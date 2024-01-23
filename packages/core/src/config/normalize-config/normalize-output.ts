import { browsersWithSupportForFeatures } from 'browserslist-generator';

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
    scriptGenTarget?: Config['config']['script']['target'];
    scriptTargets: string[] | null;
    cssTargets: string[] | null;
  } | null
>;
type TargetsMap = Omit<TargetsForTargetEnv, 'node' | 'browser'>;

const es2015Browsers = browsersWithSupportForFeatures('es6');
const es2017Browsers = browsersWithSupportForFeatures('async-functions');

const targetsMap: TargetsMap = {
  node16: {
    scriptTargets: ['node 16'],
    cssTargets: null
  },
  'node-legacy': {
    scriptTargets: ['> 0.25%, not dead'],
    cssTargets: null
  },
  'node-next': null,
  'browser-legacy': {
    scriptTargets: ['> 0.25%, not dead'],
    cssTargets: ['> 0.25%, not dead'],
    scriptGenTarget: 'es5'
  },
  'browser-es2015': {
    scriptTargets: es2015Browsers,
    cssTargets: es2015Browsers,
    scriptGenTarget: 'es2015'
  },
  'browser-es2017': {
    scriptTargets: es2017Browsers,
    cssTargets: es2017Browsers,
    scriptGenTarget: 'es2017'
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
    const { scriptTargets, cssTargets, scriptGenTarget } =
      targetsMap[targetEnv];
    // set defaults for targets
    if (config.presetEnv !== false) {
      // null means disable presetEnv
      if (scriptTargets == null) {
        config.presetEnv = false;
      } else if (typeof config.presetEnv === 'object') {
        config.presetEnv.options ??= {};
        if (!config.presetEnv.options.targets) {
          config.presetEnv.options.targets = scriptTargets;
        }
      } else {
        config.presetEnv = {
          options: {
            targets: scriptTargets
          }
        };
      }
    }

    config.script ??= { plugins: [] };
    config.script.target = scriptGenTarget ?? 'esnext';

    if (!config)
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

    config.script ??= { plugins: [] };
    config.script.target = 'esnext';

    config.css ??= {};
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
