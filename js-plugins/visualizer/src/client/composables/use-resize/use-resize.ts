import { noop } from 'foxact/noop';
import { useEffect } from 'react';

export function useResize(ref: React.RefObject<HTMLElement>, fn: () => void) {
  useEffect(() => {
    const el = ref.current;
    if (el) {
      const observer = new ResizeObserver(() => {
        fn();
      });
      observer.observe(el);
      return () => {
        observer.unobserve(el);
        observer.disconnect();
      };
    }
    return noop;
  }, [ref, fn]);
}
