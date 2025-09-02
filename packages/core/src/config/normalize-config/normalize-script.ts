import { colors, Logger } from '../../utils/index.js';
import { ResolvedCompilation } from '../types.js';

export function normalizeScript(
  resolvedCompilation: ResolvedCompilation,
  logger: Logger
) {
  if (resolvedCompilation.script?.plugins?.length) {
    logger.info(
      `Swc plugins are configured, note that Farm uses ${colors.yellow(
        'swc_core v35.0.0'
      )}, please make sure the plugin is ${colors.green('compatible')} with swc_core ${colors.yellow(
        'swc_core v35.0.0'
      )}. Otherwise, it may exit unexpectedly.`
    );
  }

  // Auto enable decorator by default when `script.decorators` is enabled
  if (resolvedCompilation.script?.decorators !== undefined) {
    if (resolvedCompilation.script.parser === undefined) {
      resolvedCompilation.script.parser = {
        esConfig: {
          decorators: true
        },
        tsConfig: {
          decorators: true
        }
      };
    } else {
      if (resolvedCompilation.script.parser.esConfig !== undefined)
        resolvedCompilation.script.parser.esConfig.decorators = true;
      else
        resolvedCompilation.script.parser.esConfig = {
          decorators: true
        };
      if (resolvedCompilation.script.parser.tsConfig !== undefined)
        resolvedCompilation.script.parser.tsConfig.decorators = true;
    }
  }
}
