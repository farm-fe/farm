import type { JsPlugin } from '@farmfe/core';
import { load } from 'cheerio';
import { injectSymbol } from './qiankun-farm-plugin-helper';

export interface PluginOptions {
  appName: string;
  devMode?: boolean;
}

const createQiankunHelper = (qiankunName: string, devMode: boolean) => `
  ${devMode ? `console.log('%c%s %c%s', 'color: green; font-weight: bold;', '[qiankun-farm-plugin]', 'color: blue;', 'injecting...')` : ''}
  const createLifeCycleBridge = (hookName) => {
    const helper = window[\`${injectSymbol}\`] || {};
    const hook = helper[\`\${hookName}\`]
    ${devMode ? `console.log('%c%s %c%s %c%s', 'color: green; font-weight: bold;', '[qiankun-farm-plugin]', 'color: blue;', 'inject lifecycle hook:', 'color: #1677ff;', hookName, hook)` : ''}
    return hook || (() => {});
  }
  const qiankunBootstrap = createLifeCycleBridge('bootstrap');
  const qiankunMount = createLifeCycleBridge('mount');
  const qiankunUnmount = createLifeCycleBridge('unmount');
  const qiankunUpdate = createLifeCycleBridge('update');

  ;(global => {
    global.qiankunName = '${qiankunName}';
    global['${qiankunName}'] = {
      bootstrap: qiankunBootstrap,
      mount: qiankunMount,
      unmount: qiankunUnmount,
      update: qiankunUpdate
    };
  })(window);
`;

export function qiankunFarmPlugin(options: PluginOptions): JsPlugin {
  return {
    name: 'qiankun-farm-plugin',
    transformHtml: {
      order: 2,
      async executor({ htmlResource }) {
        const htmlCode = Buffer.from(htmlResource.bytes).toString();
        const $ = load(htmlCode);
        $('body').append(
          `<script>${createQiankunHelper(options.appName, options?.devMode || false)}</script>`
        );
        htmlResource.bytes = [...Buffer.from($.html())];
        return htmlResource;
      }
    }
  };
}
