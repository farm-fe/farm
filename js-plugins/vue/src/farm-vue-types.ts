import { SFCDescriptor } from "@vue/compiler-sfc";

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
