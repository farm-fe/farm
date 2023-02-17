export interface HmrUpdatePacket {
  id: string;
}

export interface HmrUpdateResult {
  added: string[];
  changed: string[];
  removed: string[];

  // closest boundary modules which are related to added or changed
  boundaries: Record<string, string[][]>;
  // modules which are added or changed
  modules: Record<
    string,
    (module: any, exports: any, require: (id: string) => any) => void
  >;
}
