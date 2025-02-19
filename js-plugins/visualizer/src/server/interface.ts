import type { GroupNode } from './trie';

export interface VisualizerOptions {
  host?: string;
  port?: number;
}

export type uint8 = number;

export interface ExportFields {
  import?: string;
  require?: string;
  default?: string;
}

export interface PackageJSONMetadata {
  type: 'commonjs' | 'module';
  main?: string;
  module?: string;
  exports?: Record<string, ExportFields | string>;
  [prop: string]: any;
}

export interface HookStats {
  pluginName: string;
  hookName: string;
  moduleId: string;
  hookContext: unknown;
  input: string;
  output: string;
  duration: number;
  startTime: number;
  endTime: number;
}

export interface HookStatsMap {
  resolve: Array<HookStats>;
  transform: Array<HookStats>;
  analyze_deps: Array<HookStats>;
  optimize_module_graph: Array<HookStats>;
  render_resource_pot: Array<HookStats>;
  load: Array<HookStats>;
  process_module: Array<HookStats>;
  build_end: Array<HookStats>;
  partial_bundling: Array<HookStats>;
  process_resource_pots: Array<HookStats>;
  parse: Array<HookStats>;
  write_plugin_cache: Array<HookStats>;
  generate_resources: Array<HookStats>;
}

export interface ModuleGraphStats {
  modules: Record<string, { moduleId: string; moduleType: string }>;
  edges: Record<
    string,
    Array<[string, { source: string; kind: string; order: number }]>
  >;
}

export interface CompilationFlowStats {
  entries: Array<string>;
  hookStatsMap: HookStatsMap;
  moduleGraphStats: ModuleGraphStats;
  duration: number;
  startTime: number;
  buildEndTime: number;
  endTime: number;
}

export interface StatsMetadata {
  initialCompilationFlowStats: CompilationFlowStats;
  hmrCompilationFlowStats: CompilationFlowStats;
}

export interface AnalysisModule {
  filename: string;
  statSize: number;
  parsedSize: number;
  parsed: Array<GroupNode>;
  stats: Array<GroupNode>;
}
