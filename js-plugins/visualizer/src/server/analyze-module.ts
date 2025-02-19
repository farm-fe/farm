import path from 'node:path';
import type { Compiler, Resource } from '@farmfe/core';
import type { uint8 } from './interface';
import { byteToString, slash, stringToByte } from './shared';
import {
  pickupContentFromSourcemap,
  pickupMappingsFromCodeBinary
} from './source-map';
import { GroupNode, Trie } from './trie';

export interface SerializedModWithAssets {
  code: ArrayLike<uint8>;
  id: string;
  type: 'asset';
}

export interface SerializedModWithChunk {
  code: ArrayLike<uint8>;
  sourcemap: ArrayLike<uint8>;
  id: string;
  type: 'chunk';
}

export type SerializedMod = SerializedModWithAssets | SerializedModWithChunk;

function isSourcemap(filename: string) {
  return filename.slice(-3) === 'map';
}

function findSourcemap(
  filename: string,
  sourcemapFilename: string,
  resources: Record<string, Resource>
) {
  if (sourcemapFilename in resources) {
    return resources[sourcemapFilename].bytes;
  }
  throw new Error(
    `[@farmfe/plugin-visualizer]: Sourcemap ${sourcemapFilename} not found for ${filename}`
  );
}

export const JS_EXTENSIONS = /\.(c|m)?js$/;

const KNOWN_EXT_NAME = [
  '.mjs',
  '.js',
  '.cjs',
  '.ts',
  '.tsx',
  '.vue',
  '.svelte',
  '.md',
  '.mdx'
];

function getAbsPath(p: string, cwd: string) {
  p = slash(p);
  return p.replace(cwd, '').replace(/\0/, '');
}

function generateNodeId(id: string, cwd: string): string {
  const abs = getAbsPath(id, cwd);
  return path.isAbsolute(abs) ? abs.replace('/', '') : abs;
}

export function transformResourceMapIntoSerializedMod(
  resourcesMap: Record<string, Resource>
) {
  const result: SerializedMod[] = [];
  for (const id in resourcesMap) {
    if (isSourcemap(id)) {
      continue;
    }
    const mod = resourcesMap[id];
    if (mod.resourceType === 'js') {
      let sourcemap = [];
      if (JS_EXTENSIONS.test(id)) {
        const possiblePath = id + '.map';
        sourcemap = findSourcemap(id, possiblePath, resourcesMap);
        result.push({
          code: mod.bytes,
          sourcemap,
          id,
          type: 'chunk'
        });
      }
    } else {
      if (mod.resourceType === 'runtime') {
        continue;
      }
      result.push({
        code: mod.bytes,
        id,
        type: 'asset'
      });
    }
  }
  return result;
}

export function evaludatePluginLifecycle(c: Compiler) {
  //
}

export function evaludateModuleGraph(c: Compiler, workspaceRoot: string) {
  const serializedMod = transformResourceMapIntoSerializedMod(c.resourcesMap());
  const result: Array<VisualizerNode> = [];
  for (const mod of serializedMod) {
    const node = new VisualizerNode(mod.id);
    node.setup(mod, workspaceRoot);
    result.push(node);
  }
  return result;
}

export class VisualizerNode {
  filename: string;
  statSize: number;
  parsedSize: number;
  parsed: Array<GroupNode>;
  stats: Array<GroupNode>;
  constructor(id: string) {
    this.filename = id;
    this.statSize = 0;
    this.parsedSize = 0;
  }
  setup(mod: SerializedMod, workspaceRoot: string) {
    const stats = new Trie({ size: 0 });
    const parsed = new Trie({ size: 0 });
    if (mod.type === 'asset') {
      this.statSize = mod.code.length;
      this.parsedSize = mod.code.length;
      return;
    }
    const { code, sourcemap } = mod;
    if (!sourcemap.length) {
      return;
    }
    const infomations = pickupContentFromSourcemap(sourcemap);
    for (const info of infomations) {
      if (info.id[0] === '.') {
        info.id = path.resolve(workspaceRoot, info.id);
      }
      const statSize = stringToByte(info.code).byteLength;
      this.statSize += statSize;
      stats.insert(generateNodeId(info.id, workspaceRoot), {
        filename: info.id,
        size: statSize
      });
    }
    const { grouped, files } = pickupMappingsFromCodeBinary(
      code,
      sourcemap,
      (id) => {
        const relatived = path.relative(workspaceRoot, id);
        return path.join(workspaceRoot, relatived);
      }
    );
    if (!files.size) {
      files.add(this.filename);
      grouped[this.filename] = byteToString(code);
    }
    for (const file in grouped) {
      if (!KNOWN_EXT_NAME.includes(path.extname(file))) {
        continue;
      }
      const code = grouped[file];
      const parsedSize = stringToByte(code).byteLength;
      parsed.insert(generateNodeId(file, workspaceRoot), {
        filename: file,
        size: parsedSize
      });
      this.parsedSize += parsedSize;
    }
    stats.mergeUniqueNode();
    parsed.mergeUniqueNode();
    parsed.walk(parsed.root, {
      before: (c, p) => p.groups.push(c),
      after: (c) => {
        c.size = c.groups.reduce((acc, cur) => ((acc += cur.size), acc), 0);
      }
    });
    stats.walk(stats.root, {
      before: (c, p) => p.groups.push(c),
      after: (c) => {
        c.size = c.groups.reduce((acc, cur) => ((acc += cur.size), acc), 0);
      }
    });
    this.parsed = parsed.root.groups;
    this.stats = stats.root.groups;
  }
}

export class VisualizerModule {
  private c: Compiler | null;
  workspaceRoot: string;
  constructor() {
    this.c = null;
    this.workspaceRoot = process.cwd();
  }
  setupCompiler(c: Compiler) {
    if (!this.c) {
      this.c = c;
    }
  }
  // do analysic is designed for prepare classical treemap struct.
  doAnalysis() {
    if (!this.c) {
      throw new Error(`[@farmfe/plugin-visualizer]: Compiler isn't setup.`);
    }
    evaludatePluginLifecycle(this.c);
    evaludateModuleGraph(this.c, this.workspaceRoot);
  }
}

export function createVisualizerModule() {
  return new VisualizerModule();
}
