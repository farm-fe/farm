import { useEffect } from 'foxact/use-abortable-effect';
import type { MutableRefObject } from 'react';

const config = { attributes: false, childList: true, subtree: true };

export function useDOMObserver(
  ref: MutableRefObject<HTMLElement | null> | undefined,
  callback: MutationCallback = () => {}
) {
  useEffect(
    (signal) => {
      if (!ref || !ref.current) {
        return;
      }
      const done: MutationCallback = (...params) => {
        if (signal.aborted) {
          return;
        }
        callback(...params);
      };
      const observer = new MutationObserver(done);
      observer.observe(ref.current, config);
      return () => {
        observer.disconnect();
      };
    },
    [callback, ref]
  );
}
