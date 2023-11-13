import type { Resource } from '@farmfe/runtime/src/resource-loader';

// export interface HmrUpdatePacket {
//   id: string;
// }
type ModuleMap = Record<
  string,
  (
    module: any,
    exports: any,
    require: (id: string) => any,
    dynamicRequire: (id: string) => Promise<any>
  ) => void
>;
export interface HmrUpdateResult {
  added: string[];
  changed: string[];
  removed: string[];

  // closest boundary modules which are related to added or changed
  boundaries: Record<string, string[][]>;
  // modules which are added or changed
  modules: ModuleMap;
  dynamicResourcesMap: Record<string, Resource[]> | null;
}

export interface RawHmrUpdateResult {
  added: string[];
  changed: string[];
  removed: string[];
  boundaries: Record<string, string[][]>;
  immutableModules: string;
  mutableModules: string;
  dynamicResourcesMap: Record<string, Resource[]> | null;
}
