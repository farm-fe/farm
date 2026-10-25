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
      ...options.transformOptions,
      parserOpts: {
        plugins: ['typescript', 'jsx'],
        ...options.transformOptions?.parserOpts
      },
      plugins: [
        [
          require.resolve('babel-plugin-react-compiler'),
          options.compilerOptions ?? {}
        ],
        require.resolve('@babel/plugin-syntax-jsx'),
        [
          require.resolve('@babel/plugin-transform-typescript'),
          { isTSX: true, allowDeclareFields: true }
        ],
        ...(options.transformOptions?.plugins ?? [])
      ]
    }
  });
}
