// using new tech if target is node.
declare const __FARM_TARGET_ENV__: 'node' | 'browser';

export interface Resource {
  id: string;
  path: string;
  type: 'script' | 'link';
}

/**
 * Loading resources according to their type and target env.
 */
export class ResourceLoader {
  // static async load(resource: Resource): Promise<void> {}
}
