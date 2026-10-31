import { useEffect, useRef, useState } from 'react';
import type { ColorMode } from './config.generated.js';

export interface ColorModeState {
  colorMode: ColorMode;
  resolvedColorMode: 'light' | 'dark';
  setColorMode(mode: ColorMode): void;
}
const changed = 'farm-docs-color-mode';
function valid(value: unknown): value is ColorMode {
  return value === 'system' || value === 'light' || value === 'dark';
}

/** Browser state only: the first render deliberately matches the SSR default. */
export function useColorModeState(
  defaultMode: ColorMode,
  basePath: string
): ColorModeState {
  const key = `farm-docs:color-mode:${basePath}`;
  const [state, setState] = useState(() => ({
    colorMode: defaultMode,
    resolvedColorMode:
      defaultMode === 'dark' ? ('dark' as const) : ('light' as const)
  }));
  const selection = useRef(defaultMode);
  const media = useRef<MediaQueryList | null>(null);
  function apply(mode: ColorMode) {
    const resolved =
      mode === 'system' ? (media.current?.matches ? 'dark' : 'light') : mode;
    selection.current = mode;
    document.documentElement.dataset.theme = resolved;
    document.documentElement.style.colorScheme = resolved;
    setState({ colorMode: mode, resolvedColorMode: resolved });
  }
  useEffect(() => {
    let saved: unknown;
    try {
      saved = localStorage.getItem(key);
    } catch {
      /* Persistence is optional. */
    }
    try {
      media.current = matchMedia('(prefers-color-scheme: dark)');
    } catch {
      media.current = null;
    }
    apply(valid(saved) ? saved : defaultMode);
    const system = () => {
      if (selection.current === 'system') apply('system');
    };
    const storage = (event: StorageEvent) => {
      // Ignore sessionStorage and unrelated site preferences, but accept clear().
      let area: Storage | undefined;
      try {
        area = localStorage;
      } catch {
        return;
      }
      if (
        event.storageArea !== area ||
        (event.key !== null && event.key !== key)
      )
        return;
      apply(valid(event.newValue) ? event.newValue : defaultMode);
    };
    const sibling = (event: Event) => {
      const detail = (event as CustomEvent<{ key: string; mode: unknown }>)
        .detail;
      if (detail?.key === key && valid(detail.mode)) apply(detail.mode);
    };
    media.current?.addEventListener('change', system);
    window.addEventListener('storage', storage);
    window.addEventListener(changed, sibling);
    return () => {
      media.current?.removeEventListener('change', system);
      window.removeEventListener('storage', storage);
      window.removeEventListener(changed, sibling);
      media.current = null;
    };
  }, [defaultMode, key]);
  return {
    ...state,
    setColorMode(mode) {
      if (!valid(mode))
        throw new Error('[DOCS_COLOR_MODE] expected system, light or dark');
      apply(mode);
      try {
        localStorage.setItem(key, mode);
      } catch {
        /* A blocked write cannot block theme changes. */
      }
      window.dispatchEvent(new CustomEvent(changed, { detail: { key, mode } }));
    }
  };
}
