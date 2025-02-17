import type { JsPlugin } from '@farmfe/core';
import { createVisualizerModule } from './analyze-module';
import { createInternalServices } from './api';
import type { VisualizerOptions } from './interface';

const defaultOptions = {} satisfies VisualizerOptions;

export function visualizer(opts?: VisualizerOptions) {
  const options = { ...defaultOptions, ...opts };
  const visualizerModule = createVisualizerModule();
  const services = createInternalServices();
  const plugin = <JsPlugin>{
    name: '@farmfe/plugin-visualizer',
    config(conf, env) {
      if (env.command === 'dev') {
        conf.compilation.record = true;
      }

      return conf;
    },
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (req.url.startsWith('/__visualizer')) {
          return services.handler(req, res, next);
        }
        next();
      });
    },
    configureCompiler(compiler) {
      visualizerModule.setupCompiler(compiler);
    },
    finish: {
      executor(param, context, hookContext) {
        // services.server.listen(8888, () => {});
        console.log(param);
      }
    }
  };

  return plugin;
}
