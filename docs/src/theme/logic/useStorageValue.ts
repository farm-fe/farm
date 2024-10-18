import { useCallback, useEffect, useState } from 'react';

/**
 * Read/update the value in localStorage, and keeping it in sync with other tabs.
 */
export const useStorageValue = (key: string, defaultValue = null) => {
  const [value, setValueInternal] = useState(() => {
    if (typeof window === 'undefined') {
      return defaultValue;
    }
    return localStorage.getItem(key) ?? defaultValue;
  });

  const setValue = useCallback(
    value => {
      setValueInternal(prev => {
        const next = typeof value === 'function' ? value(prev) : value;
        if (next == null) {
          localStorage.removeItem(key);
        } else {
          localStorage.setItem(key, next);
        }
        return next;
      });
    },
    [key],
  );

  useEffect(() => {
    const listener = (e: StorageEvent) => {
      if (e.key === key) {
        setValueInternal(localStorage.getItem(key) ?? defaultValue);
      }
    };
    window.addEventListener('storage', listener);
    return () => {
      window.removeEventListener('storage', listener);
    };
  }, [key, defaultValue]);

  return [value, setValue] as const;
};
