import Less from 'less';
import Sass from 'sass';
import Stylus from 'stylus';
import { SFCDescriptor } from '@vue/compiler-sfc';

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
