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
