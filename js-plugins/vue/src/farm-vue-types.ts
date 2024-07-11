import type {
  SFCDescriptor,
  SFCScriptCompileOptions,
  SFCStyleCompileOptions,
  SFCTemplateCompileOptions
} from '@vue/compiler-sfc';
import type Less from 'less';
import type Sass from 'sass';
import type Stylus from 'stylus';

export interface outputData {
  id: string;
  message: string | object;
}
export interface QueryObj {
  [key: string]: string | number | boolean;
}
export interface StylesCodeCache {
  [key: string]: string;
}

export type CacheDescriptor = Record<string, SFCDescriptor>;

export type Union<A, B> = A & B;

export type ValueOf<T> = T[keyof T];

export type PreProcessors = {
  [PreProcessorsType.less]: typeof Less;
  [PreProcessorsType.sass]: typeof Sass;
  [PreProcessorsType.stylus]: typeof Stylus;
};

export enum PreProcessorsType {
  less = 'less',
  sass = 'sass',
  stylus = 'stylus'
}

export type PreProcessorsOptions<T> = T extends typeof Less
  ? Less.Options
  : T extends typeof Sass
    ? Sass.Options<'async'>
    : T extends typeof Stylus
      ? Stylus.RenderOptions
      : never;

export interface FarmVuePluginOptions {
  include?: string | RegExp | (string | RegExp)[];
  exclude?: string | RegExp | (string | RegExp)[];
  isProduction?: boolean;
  sourceMap?: boolean;
  script?: Partial<Pick<SFCScriptCompileOptions, 'babelParserPlugins'>>;
  template?: Partial<
    Pick<
      SFCTemplateCompileOptions,
      | 'compiler'
      | 'compilerOptions'
      | 'preprocessOptions'
      | 'preprocessCustomRequire'
      | 'transformAssetUrls'
    >
  >;
  style?: Partial<Pick<SFCStyleCompileOptions, 'trim'>>;
  /**
   * If not set, it is considered to be `true` in `development` mode.
   */
  hmr?: boolean;

  /**
   * When set to `true`, it will disable `compilation.lazyCompilation` and `server.hmr`.
   */
  ssr?: boolean;
}

export interface ResolvedOptions {
  include: (string | RegExp)[];
  exclude: (string | RegExp)[];
  isProduction: boolean;
  sourceMap: boolean;
  script: Partial<Pick<SFCScriptCompileOptions, 'babelParserPlugins'>>;
  template: Partial<
    Pick<
      SFCTemplateCompileOptions,
      | 'compiler'
      | 'compilerOptions'
      | 'preprocessOptions'
      | 'preprocessCustomRequire'
      | 'transformAssetUrls'
    >
  >;
  style: Partial<Pick<SFCStyleCompileOptions, 'trim'>>;
  hmr?: boolean;
  ssr: boolean;
}
