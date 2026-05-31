import type { JsPlugin } from '@farmfe/core';
import { type BabelOptions, babel } from '@farmfe/js-plugin-babel';
import type { PluginOptions } from './types';

interface ReactCompilerOptions
  extends Pick<BabelOptions, 'filters' | 'transformOptions'> {
  compilerOptions?: Partial<PluginOptions>;
}

const defaultReactCompilerModuleTypes = ['jsx', 'tsx'] as const;
const defaultReactCompilerResolvedPaths: string[] = [];

export function reactCompiler(options: ReactCompilerOptions = {}): JsPlugin {
  return babel({
    name: 'js-plugin:react-compiler',
    priority: 120,
    filters: {
      moduleTypes: options.filters?.moduleTypes ?? [
        ...defaultReactCompilerModuleTypes
      ],
      resolvedPaths:
        options.filters?.resolvedPaths ?? defaultReactCompilerResolvedPaths
    },
    transformOptions: {
      plugins: ['babel-plugin-react-compiler', '@babel/plugin-syntax-jsx'].map(
        (pkg) => require.resolve(pkg)
      )
    }
  });
}
