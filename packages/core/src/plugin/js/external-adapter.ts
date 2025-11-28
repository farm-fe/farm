import {
  mergeCustomExternal,
  partialExternal
} from '../../config/normalize-config/normalize-external.js';
import { UserConfig } from '../../config/types.js';
import { isArray } from '../../utils/share.js';
import { JsPlugin } from '../type.js';

/**
 * avoid add new external in config hook
 */
export function externalAdapter(): JsPlugin {
  return {
    name: 'farm:external-adapter',

    priority: -Infinity,

    config(config: UserConfig): UserConfig | Promise<UserConfig> {
      if (
        config?.compilation?.external &&
        isArray(config.compilation.external)
      ) {
        let [stringExternal, recordExternal] = mergeCustomExternal(
          partialExternal(config.compilation.external)
        );

        return {
          compilation: {
            external: stringExternal,
            output: {
              externalGlobals: recordExternal
            }
          }
        };
      }
    }
  };
}
