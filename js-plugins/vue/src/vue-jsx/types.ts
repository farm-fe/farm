import type { VueJSXPluginOptions } from '@vue/babel-plugin-jsx';

export type Options = VueJSXPluginOptions & { babelPlugins?: any[] } & {
  include?: (string | RegExp)[] | (string | RegExp);
  exclude?: (string | RegExp)[] | (string | RegExp);
};
