import { browsersWithSupportForFeatures } from 'farm-browserslist-generator';

import path, { isAbsolute } from 'node:path';
import { Config } from '../../types/binding.js';
import { urlRegex } from '../../utils/http.js';
import { Logger } from '../../utils/logger.js';
import {
  FARM_TARGET_BROWSER_ENVS,
  mapTargetEnvValue,
  normalizeBasePath
} from '../../utils/share.js';
import { ResolvedCompilation } from '../types.js';

export function normalizeOutput(
  config: ResolvedCompilation,
  isProduction: boolean,
  logger: Logger
) {
  if (!config.output) {
    config.output = {};
  }

  if (!config.output.targetEnv) {
    config.output.targetEnv = 'browser';
  }

  if (isProduction) {
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

  // resolve public path
  config.output.publicPath = normalizePublicPath(
    config.output.targetEnv,
    config.output?.publicPath,
    logger
  );
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

export const targetEnvMapPlatform: Record<string, string> = {
  'lib-node': 'node',
  'lib-browser': 'browser'
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
          config.input &&
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

function tryGetDefaultPublicPath(
  targetEnv: string,
  publicPath: string | undefined,
  logger: Logger
): string | undefined {
  if (!targetEnv) {
    return publicPath;
  }

  if (publicPath) {
    if (urlRegex.test(publicPath)) {
      return publicPath;
    }

    if (targetEnv === 'node' && isAbsolute(publicPath)) {
      // vitejs plugin maybe set absolute path, should transform to relative path
      const relativePath = './' + path.posix.normalize(publicPath).slice(1);

      logger.warn(
        `publicPath can't support absolute path in NodeJs, will be transform "${publicPath}" to "${relativePath}".`
      );

      return relativePath;
    }

    return publicPath;
  }

  if (['node', 'browser'].includes(targetEnv)) {
    return targetEnv === 'node' ? './' : '/';
  }
}

/**
 * @param outputConfig  publicPath option
 * @param logger  logger instance
 * @param isPrefixNeeded  whether to add a prefix to the publicPath
 * @returns  normalized publicPath
 */
export function normalizePublicPath(
  targetEnv: string,
  publicPath: string | undefined,
  logger: Logger,
  isPrefixNeeded = true
) {
  let defaultPublicPath =
    tryGetDefaultPublicPath(targetEnv, publicPath, logger) ?? '/';

  let warning = false;
  // ../ or ../xxx warning
  // normalize relative path
  if (defaultPublicPath.startsWith('..')) {
    warning = true;
  }

  // . ./xx => ./ ./xx/
  // normalize appended relative path
  if (!defaultPublicPath.endsWith('/')) {
    if (!urlRegex.test(defaultPublicPath)) {
      warning = true;
    }
    defaultPublicPath = defaultPublicPath + '/';
  }

  // normalize prepended relative path
  if (
    defaultPublicPath.startsWith('/') &&
    !urlRegex.test(defaultPublicPath) &&
    !isPrefixNeeded
  ) {
    defaultPublicPath = defaultPublicPath.slice(1);
  }

  warning &&
    isPrefixNeeded &&
    logger.warn(
      ` (!) Irregular 'publicPath' options: '${publicPath}', it should only be an absolute path like '/publicPath/', './', an url or an empty string.`
    );

  return defaultPublicPath;
}

export function getValidPublicPath(publicPath = '/'): string | undefined {
  let validPublicPath;

  if (publicPath.startsWith('/')) {
    validPublicPath = publicPath;
  } else if (publicPath.startsWith('.')) {
    validPublicPath = normalizeBasePath(path.join('/', publicPath));
  }

  return validPublicPath;
}
