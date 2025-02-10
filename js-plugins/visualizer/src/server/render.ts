import type { Compiler, Resource } from '@farmfe/core';
import { transformResourceInfo2RollupResource } from '@farmfe/core';
import type { OutputBundle } from 'rollup';
import { createServer } from 'vite-bundle-analyzer';
import type { Module } from 'vite-bundle-analyzer';
import { createAnalyzerModule } from 'vite-bundle-analyzer/sdk/server';
// core module provide a adapter for rollup like tool
// If one day this part is removed, we should copy it from core module

class VisualizerModule {
  private c: Compiler | null;
  private nativeModule: ReturnType<typeof createAnalyzerModule>;
  private chunks: OutputBundle | null;
  private resourceMap: Record<string, Resource>;
  workspaceRoot: string;
  constructor() {
    this.c = null;
    this.workspaceRoot = process.cwd();
    this.nativeModule = createAnalyzerModule();
    this.chunks = null;
  }
  setupCompiler(c: Compiler) {
    if (!this.c) {
      this.c = c;
    }
  }
  async doAnalysis() {
    if (!this.c) {
      throw new Error('[farm-visualizer] compiler is not setup yet');
    }
    this.resourceMap = this.c.resourcesMap();
    this.chunks = Object.entries(this.resourceMap).reduce(
      (acc, [id, resource]) => {
        acc[id] = transformResourceInfo2RollupResource(resource);
        return acc;
      },
      {} as OutputBundle
    );
    this.nativeModule.setupPluginContextPolyfill({
      getModuleInfo: (id) => {
        console.log(id);
        return { code: '', id: '' };
      }
    });
    this.nativeModule.setupRollupChunks(this.chunks);
    for (const bundleName in this.chunks) {
      const bundle = this.chunks[bundleName];
      await this.nativeModule.addModule(bundle);
    }
  }
  process(): Module[] {
    return this.nativeModule.processModule();
  }
}

export function createVisualizerModule() {
  return new VisualizerModule();
}

export function createVisualizerServer() {
  const server = createServer();
}
