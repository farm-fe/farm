/**
 * Hack plugin to transform css for Vite compatibility:
 * 1. wrap the css code with `` before post plugins execute(priority 98)
 * 2. unwrap the css code from `` after post plugins execute
 */

import { JsPlugin } from '../../type.js';
import { normalizeFilterPath } from '../utils.js';

const BEGIN = '__farm_vite_css_hack_start__=`';
const END = '`;__farm_vite_css_hack_end__';

export function cssPluginWrap(options: {
  filtersUnion: Set<string>;
}): JsPlugin {
  const { filtersUnion } = options;
  const resolvedPaths = Array.from(filtersUnion).map(normalizeFilterPath);

  return {
    name: 'vite-adapter-css-plugin-wrap',
    priority: 98,
    transform: {
      filters: {
        resolvedPaths,
        moduleTypes: []
      },
      async executor(param) {
        if (param.moduleType === 'css') {
          return {
            content: BEGIN + param.content + END
          };
        }
      }
    }
  };
}

export function cssPluginUnwrap(options: {
  filtersUnion: Set<string>;
}): JsPlugin {
  const { filtersUnion } = options;
  const resolvedPaths = Array.from(filtersUnion).map(normalizeFilterPath);

  return {
    name: 'vite-adapter-css-plugin-unwrap',
    priority: 98,
    transform: {
      filters: {
        resolvedPaths,
        moduleTypes: []
      },
      async executor(param) {
        if (param.moduleType === 'css') {
          return {
            content: param.content.replace(BEGIN, '').replace(END, '')
          };
        }
      }
    }
  };
}
