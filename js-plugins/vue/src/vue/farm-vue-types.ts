import Less from 'less';
import Sass from 'sass';
import Stylus from 'stylus';
import {
  SFCDescriptor,
  SFCScriptCompileOptions,
  SFCTemplateCompileOptions,
  SFCStyleCompileOptions,
} from '@vue/compiler-sfc';

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
  stylus = 'stylus',
}

export type PreProcessorsOptions<T> = T extends typeof Less
  ? Less.Options
  : T extends typeof Sass
  ? Sass.Options
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
}
