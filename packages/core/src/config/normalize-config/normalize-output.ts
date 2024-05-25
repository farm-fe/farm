import { browsersWithSupportForFeatures } from 'browserslist-generator';

import { Config } from '../../types/binding.js';
import {
  FARM_TARGET_BROWSER_ENVS,
  mapTargetEnvValue
} from '../../utils/share.js';
import { ResolvedCompilation } from '../types.js';

export async function normalizeOutput(
  config: ResolvedCompilation,
  isProduction: boolean
) {
  if (!config.output.targetEnv) {
    config.output.targetEnv = 'browser';
  }

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

  // only set default polyfill in production
  if (isProduction) {
    normalizeTargetEnv(config);
  }

  // the rust compiler only receives 'node' or 'browser'.
  mapTargetEnvValue(config);
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
const LEGACY_BROWSERS = ['ie >= 9'];

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
    scriptTargets: LEGACY_BROWSERS,
    cssTargets: LEGACY_BROWSERS,
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
 */
function normalizeTargetEnv(config: Config['config']) {
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
        if (
          FARM_TARGET_BROWSER_ENVS.includes(targetEnv) &&
          Object.values(config.input).some((v) => v?.endsWith('.html'))
        ) {
          config.presetEnv = {
            options: {
              targets: scriptTargets
            }
          };
        } else {
          config.presetEnv = false;
        }
      }
    }

    config.script ??= { plugins: [] };
    config.script.target = config.script.target ?? scriptGenTarget ?? 'esnext';

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
}
