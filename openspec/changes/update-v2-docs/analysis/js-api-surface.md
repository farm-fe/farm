# Farm v2 Codebase Analysis - JavaScript/TypeScript API Surface

## Main Export Entry Point

**Location**: `packages/core/src/index.ts`

### Core Exports

```typescript
// Main functions
export { start } - Start development server
export { preview } - Start preview server  
export { build } - Build for production

// Compiler
export { createCompiler, Compiler } from './compiler/index.js';

// Config
export { resolveConfig, UserConfig, defineFarmConfig } from './config/index.js';
export { ResolvedUserConfig, FarmCliOptions } from './config/types.js';

// Plugin system
export * from './plugin/index.js';
export * from './plugin/type.js';

// Server
export { Server, PreviewServer } from './server/index.js';

// Utilities
export * from './utils/index.js';

// Bindings
export { Resolver } from '../binding/index.js';
```

## Plugin System Types

**Location**: `packages/core/src/plugin/type.ts`

### Main Plugin Interface

```typescript
export interface JsPlugin {
  name: string;
  priority?: number;

  // Configuration hooks
  config?: (config: UserConfig, configEnv: ConfigEnv) => UserConfig | Promise<UserConfig>;
  configResolved?: (config: ResolvedUserConfig) => void | Promise<void>;
  configureServer?: (server: Server) => void | Promise<void>;  // dev mode only
  configureCompiler?: (compiler: Compiler) => void | Promise<void>;

  // Build lifecycle hooks
  buildStart?: { executor: Callback<Record<string, never>, void> };
  buildEnd?: { executor: Callback<Record<string, never>, void> };
  finish?: { executor: Callback<Record<string, never>, void> };

  // Module hooks with filters
  resolve?: JsPluginHook<
    { importers: string[]; sources: string[]; },
    PluginResolveHookParam,
    PluginResolveHookResult
  >;
  load?: JsPluginHook<
    { resolvedPaths: string[] },
    PluginLoadHookParam,
    PluginLoadHookResult
  >;
  transform?: JsPluginHook<
    { resolvedPaths?: string[]; moduleTypes?: string[] },
    PluginTransformHookParam,
    PluginTransformHookResult
  >;
  processModule?: JsPluginHook<
    NormalizeFilterParams,
    PluginProcessModuleParams,
    PluginProcessModuleResult
  >;
  freezeModule?: JsPluginHook<
    NormalizeFilterParams,
    FreeModuleParam,
    PluginProcessModuleResult
  >;

  // Resource hooks
  renderStart?: { executor: Callback<Config['config'], void> };
  processRenderedResourcePot?: JsPluginHook<
    { resourcePotTypes?: ResourcePotType[]; moduleIds?: string[]; },
    JsResourcePot,
    PluginRenderResourcePotResult
  >;
  augmentResourcePotHash?: JsPluginHook<
    { resourcePotTypes?: ResourcePotType[]; moduleIds?: string[]; },
    JsResourcePot,
    string
  >;
  finalizeResources?: {
    executor: Callback<PluginFinalizeResourcesHookParams, PluginFinalizeResourcesHookParams['resourcesMap']>;
  };
  
  // HTML transformation
  transformHtml?: {
    order?: 0 | 1 | 2;  // 0: pre, 1: normal, 2: post
    executor: Callback<{ htmlResource: Resource }, Resource>;
  };

  // I/O hooks
  writeResources?: {
    executor: (param: PluginFinalizeResourcesHookParams) => void | Promise<void>;
  };

  // Cache hooks
  pluginCacheLoaded?: { executor: Callback<number[], undefined | null | void> };
  writePluginCache?: { executor: Callback<undefined, number[]> };

  // HMR hooks
  updateModules?: {
    executor: Callback<PluginUpdateModulesHookParam, [string, UpdateType][] | undefined | null | void>;
  };
  updateFinished?: { executor: Callback<Record<string, never>, void> };
}

// Helper type for hooks with filters
type JsPluginHook<F, P, R> = { 
  filters: F; 
  executor: Callback<P, R> 
};
```

### Key Plugin Types

```typescript
export interface CompilationContext {
  resolve(param: PluginResolveHookParam, hookContext): Promise<PluginResolveHookResult>;
  addWatchFile(currentFile: string, targetFile: string): void;
  emitFile(params: CompilationContextEmitFileParams): void;
  getWatchFiles(): string[];
  warn(message: string): void;
  error(message: string): void;
  sourceMapEnabled(id: string): boolean;
  viteGetModulesByFile(file: string): ViteModule[];
  viteGetModuleById(id: string): ViteModule;
  viteGetImporters(file: string): ViteModule[];
}

export interface PluginResolveHookParam {
  source: string;
  importer: string | null;
  kind: ResolveKind;
}

export interface PluginResolveHookResult {
  resolvedPath: string;
  external?: boolean;
  sideEffects?: boolean;
  query?: [string, string][];
  meta?: Record<string, string>;
}

export interface PluginLoadHookParam {
  moduleId: string;
  resolvedPath: string;
  query: [string, string][];
  meta: Record<string, string>;
}

export interface PluginLoadHookResult {
  content: string;
  moduleType?: ModuleType;
  sourceMap?: string;
}

export interface PluginTransformHookParam {
  content: string;
  moduleId: string;
  moduleType: ModuleType;
  resolvedPath: string;
  query: [string, string][];
  meta: Record<string, string>;
}

export interface PluginTransformHookResult {
  content: string;
  moduleType?: ModuleType;
  sourceMap?: string;
  ignorePreviousSourceMap?: boolean;
}
```

## Configuration Types

**Location**: `packages/core/src/config/types.ts` and `packages/core/src/types/binding.ts`

### User Config Interface

```typescript
export interface UserConfig {
  root?: string;
  configPath?: string;
  compilation?: {
    input?: Record<string, string>;
    output?: OutputConfig;
    resolve?: ResolveConfig;
    mode?: 'development' | 'production';
    // ... many more options
  };
  server?: ServerConfig;
  plugins?: (JsPlugin | RustPlugin | vitePluginAdapter)[];
}
```

## Packages Structure

- `core` - Main package with compiler, config, plugins, server
- `cli` - Command-line interface
- `runtime` - Runtime code for browser
- `runtime-plugin-hmr` - HMR runtime plugin
- `runtime-plugin-import-meta` - Import meta runtime plugin
- `plugin-tools` - Tools for plugin development
- `utils` - Shared utilities
- `create-farm` - Project scaffolding
- `create-farm-plugin` - Plugin scaffolding

## Next Steps

- Document binding types from Rust
- Analyze dev server configuration
- Document HMR API
