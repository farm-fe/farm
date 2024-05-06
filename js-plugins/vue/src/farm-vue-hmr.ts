import {
  SFCDescriptor,
  SFCScriptBlock,
  SFCStyleBlock,
  SFCTemplateBlock
} from '@vue/compiler-sfc';
import {
  CacheDescriptor,
  QueryObj,
  ResolvedOptions,
  StylesCodeCache
} from './farm-vue-types.js';
import { genMainCode } from './generatorCode.js';

export const cacheScript = new WeakMap();

export function handleHmr(
  resolvedOptions: ResolvedOptions,
  beforeDescriptor: SFCDescriptor,
  descriptor: SFCDescriptor,
  stylesCodeCache: StylesCodeCache,
  resolvedPath: string
) {
  return diffDescriptor(
    resolvedOptions,
    beforeDescriptor,
    descriptor,
    stylesCodeCache,
    resolvedPath
  );
}

function diffDescriptor(
  resolvedOptions: ResolvedOptions,
  prevDescriptor: SFCDescriptor,
  descriptor: SFCDescriptor,
  stylesCodeCache: StylesCodeCache,
  resolvedPath: string
) {
  let _rerender_only = false;
  // if script changed, rerender from root.
  const scriptChanged = hasScriptChanged(prevDescriptor, descriptor);
  // if only template changed, rerender from current node.
  const templateChanged = hasTemplateChanged(
    prevDescriptor.template!,
    descriptor.template!
  );
  // if style changed, insert new style.
  const [deleteStyles, addStyles] = hasStyleChanged(
    prevDescriptor.styles || [],
    descriptor.styles || []
  );

  if (!scriptChanged && templateChanged) {
    _rerender_only = true;
  }

  const { source, moduleType, map } = genMainCode(
    resolvedOptions,
    descriptor,
    stylesCodeCache,
    resolvedPath,
    true,
    _rerender_only,
    deleteStyles,
    addStyles
  );

  return { source, moduleType, map };
}

function hasStyleChanged(prev: SFCStyleBlock[], next: SFCStyleBlock[]) {
  let p = 0,
    q = 0;
  const deleteStyles: SFCStyleBlock[] = [];
  const addStyles: SFCStyleBlock[] = [];
  while (prev[p] && next[q]) {
    const prevStyle = prev[p++];
    const nextStyle = next[q++];
    if (isEqualBlock(prevStyle, nextStyle)) {
      continue;
    } else {
      deleteStyles.push(prevStyle);
      addStyles.push(nextStyle);
    }
  }
  // prev should be delete
  if (prev[p] && !next[q]) {
    while (prev[p]) {
      deleteStyles.push(prev[p++]);
    }
  }
  // next has more new styles
  else if (!prev[p] && next[q]) {
    while (next[q]) {
      addStyles.push(next[q++]);
    }
  }

  return [deleteStyles, addStyles];
}

function hasTemplateChanged(prev: SFCTemplateBlock, next: SFCTemplateBlock) {
  return isEqualBlock(prev, next);
}

function hasScriptChanged(prev: SFCDescriptor, next: SFCDescriptor) {
  if (!isEqualBlock(prev.script!, next.script!)) {
    return true;
  }
  if (!isEqualBlock(prev.scriptSetup!, next.scriptSetup!)) {
    return true;
  }
  // if cssVars changed, it means that script changed
  if (prev.cssVars.join('') !== next.cssVars.join('')) {
    return true;
  }
  const prevResolvedScript = cacheScript.get(prev);
  const prevImports = prevResolvedScript?.imports;
  if (prevImports) {
    return !next.template || next.shouldForceReload(prevImports);
  }
  return false;
}

export function isEqualBlock(
  a: SFCScriptBlock | SFCTemplateBlock | SFCStyleBlock,
  b: SFCScriptBlock | SFCTemplateBlock | SFCStyleBlock
) {
  if (!a && !b) return true;
  if (!a || !b) return false;
  if (a.src && b.src && a.src === b.src) return true;
  if (a.content !== b.content) return false;
  const keysA = Object.keys(a.attrs);
  const keysB = Object.keys(b.attrs);
  if (keysA.length !== keysB.length) {
    return false;
  }
  return keysA.every((key) => a.attrs[key] === b.attrs[key]);
}
