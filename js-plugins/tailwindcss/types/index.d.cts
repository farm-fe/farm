type JsPlugin = import('@farmfe/core', { with: { 'resolution-mode': 'import' } }).JsPlugin;

export interface Options {
  filters?: {
    resolvedPaths?: string[];
    moduleTypes?: string[];
  };
}

export default function tailwindcss(options?: Options): JsPlugin[];