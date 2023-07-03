import { JsPlugin, UserConfig } from '@farmfe/core';
import postcssLoadConfig, { ProcessOptionsPreload } from 'postcss-load-config';
import { Processor } from 'postcss';
import module from 'module';

const require = module.createRequire(__dirname);

export type PostcssPluginOptions = ProcessOptionsPreload & { map?: boolean };

export default function farmPostcssPlugin(
  pluginOptions: PostcssPluginOptions = {}
): JsPlugin {
  let postcssProcessor: Processor;
  let postcssOption: any;

  return {
    name: 'farm-js-plugin-postcss',
    // Execute last
    priority: 0,
    config: async (config: UserConfig) => {
      const { plugins, options } = await postcssLoadConfig(
        pluginOptions,
        config.root
      );
      const packageName = 'postcss';
      const postcss = require(packageName);
      postcssProcessor = postcss(plugins);
      postcssOption = options;
      return config;
    },
    transform: {
      filters: { moduleTypes: ['css'] },
      async executor(param) {
        const { css, map } = await postcssProcessor.process(param.content, {
          ...postcssOption,
          from: param.resolvedPath
        });

        return {
          content: css,
          moduleType: 'css',
          sourceMap: pluginOptions.map && JSON.stringify(map.toJSON())
        };
      }
    }
  };
}
