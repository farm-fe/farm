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

// the same as Vite, see LICENSE. modified by @farmfe
export type HMRPayload =
  | FarmHmrPayload
  | ConnectedPayload
  | UpdatePayload
  | FullReloadPayload
  | CustomPayload
  | ErrorPayload
  | PrunePayload
  | ClosingPayload;

export interface FarmHmrPayload {
  type: 'farm-update';
  result: RawHmrUpdateResult;
}

export interface ConnectedPayload {
  type: 'connected';
}

export interface UpdatePayload {
  type: 'update';
  updates: Update[];
}

export interface Update {
  type: 'js-update' | 'css-update';
  path: string;
  acceptedPath: string;
  timestamp: number;
  /**
   * @experimental internal
   */
  explicitImportRequired?: boolean | undefined;
}

export interface PrunePayload {
  type: 'prune';
  paths: string[];
}

export interface FullReloadPayload {
  type: 'full-reload';
  path?: string;
}

export interface CustomPayload {
  type: 'custom';
  event: string;
  data?: any;
}

export interface ClosingPayload {
  type: 'closing';
}

export interface ErrorPayload {
  type: 'error';
  err: {
    [name: string]: any;
    message: string;
    stack: string;
    id?: string;
    frame?: string;
    plugin?: string;
    pluginCode?: string;
    loc?: {
      file?: string;
      line: number;
      column: number;
    };
  };
  overlay: boolean;
}
