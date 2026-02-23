import { cleanUrl } from '../utils/url.js';
import type { ResolvedFetchResult } from './types.js';

export class EvaluatedModuleNode {
  importers: Set<string> = new Set();
  imports: Set<string> = new Set();
  evaluated = false;
  meta: ResolvedFetchResult | undefined;
  promise: Promise<unknown> | undefined;
  exports: unknown;

  constructor(
    public id: string,
    public url: string
  ) {}
}

export class EvaluatedModules {
  readonly idToModuleMap = new Map<string, EvaluatedModuleNode>();
  readonly urlToModuleMap = new Map<string, EvaluatedModuleNode>();
  readonly fileToModulesMap = new Map<string, Set<EvaluatedModuleNode>>();

  getModuleById(id: string): EvaluatedModuleNode | undefined {
    return this.idToModuleMap.get(id);
  }

  getModuleByUrl(url: string): EvaluatedModuleNode | undefined {
    return this.urlToModuleMap.get(url);
  }

  getModulesByFile(file: string): Set<EvaluatedModuleNode> | undefined {
    return this.fileToModulesMap.get(cleanUrl(file));
  }

  ensureModule(id: string, url: string): EvaluatedModuleNode {
    const existing = this.idToModuleMap.get(id);

    if (existing) {
      this.urlToModuleMap.set(url, existing);
      return existing;
    }

    const mod = new EvaluatedModuleNode(id, url);
    this.idToModuleMap.set(id, mod);
    this.urlToModuleMap.set(url, mod);

    const fileKey = cleanUrl(id);
    const set =
      this.fileToModulesMap.get(fileKey) ?? new Set<EvaluatedModuleNode>();
    set.add(mod);
    this.fileToModulesMap.set(fileKey, set);

    return mod;
  }

  invalidateModule(mod: EvaluatedModuleNode): void {
    // detach reverse edges so stale dependency links don't accumulate across updates
    for (const depId of mod.imports) {
      this.idToModuleMap.get(depId)?.importers.delete(mod.id);
    }

    for (const importerId of mod.importers) {
      this.idToModuleMap.get(importerId)?.imports.delete(mod.id);
    }

    mod.evaluated = false;
    mod.promise = undefined;
    mod.exports = undefined;
    mod.meta = undefined;
    mod.imports.clear();
  }

  clear(): void {
    this.idToModuleMap.clear();
    this.urlToModuleMap.clear();
    this.fileToModulesMap.clear();
  }
}
