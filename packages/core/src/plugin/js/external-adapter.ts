import { CUSTOM_KEYS } from '../../config/constants';
import {
  mergeCustomExternal,
  partialExternal
} from '../../config/normalize-config/normalize-external';
import { UserConfig } from '../../config/types';
import { isArray } from '../../utils/share';
import { JsPlugin } from '../type';

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
          config?.compilation,
          partialExternal(config.compilation.external)
        );

        return {
          compilation: {
            external: stringExternal,
            custom: {
              [CUSTOM_KEYS.external_record]: JSON.stringify(recordExternal)
            }
          }
        };
      }
    }
  };
}
