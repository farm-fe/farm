import { useEffect } from 'react';

export function useClickAnyWhere(
  handler: (this: Document, ev: MouseEvent) => void
) {
  useEffect(() => {
    document.addEventListener('click', handler);
    return () => document.removeEventListener('click', handler);
  }, [handler]);
}
