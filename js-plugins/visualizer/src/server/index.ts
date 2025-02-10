import type { JsPlugin } from '@farmfe/core';
import type { VisualizerOptions } from './interface';
import { createVisualizerModule, createVisualizerServer } from './render';

const defaultOptions = {} satisfies VisualizerOptions;

export function visualizer(opts?: VisualizerOptions) {
  const options = { ...defaultOptions, ...opts };
  const visualizerModule = createVisualizerModule();
  const ctx = <JsPlugin>{
    name: 'farm-visualizer',
    config(conf) {
      if (!conf.compilation) {
        conf.compilation = {};
      }
      conf.compilation.record = true;
      visualizerModule.workspaceRoot = conf.root;
      return conf;
    },
    async configureCompiler(compiler) {
      visualizerModule.setupCompiler(compiler);
      await visualizerModule.doAnalysis();
    },
    finish: {
      executor(param, context, hookContext) {
        const server = createVisualizerServer();
      }
    }
  };

  return ctx;
}
