import type { Compiler, Resource } from '@farmfe/core';
import { createServer as _createServer } from 'vite-bundle-analyzer';

export class VisualizerModule {
  private c: Compiler | null;
  constructor() {
    this.c = null;
  }
  setupCompiler(c: Compiler) {
    if (!this.c) {
      this.c = c;
    }
  }
  doAnalysis() {}
}

export function createVisualizerModule() {
  return new VisualizerModule();
}

export function createServer() {
  const app = _createServer();
}
