import type { MutableRefObject } from 'react';

export function getElementOffset(el?: HTMLElement | null) {
  if (!el) {
    return {
      top: 0,
      left: 0
    };
  }
  const { top, left } = el.getBoundingClientRect();
  return { top, left };
}

export interface ReactiveDomReact {
  top: number;
  left: number;
  right: number;
  width: number;
  height: number;
  elementTop: number;
}
const defaultRect: ReactiveDomReact = {
  top: -1000,
  left: -1000,
  right: -1000,
  width: 0,
  height: 0,
  elementTop: -1000
};

function getRectFromDOMWithContainer(
  domRect?: DOMRect,
  getContainer?: () => HTMLElement | null
): ReactiveDomReact {
  if (!domRect) {
    return defaultRect;
  }
  const container = getContainer ? getContainer() : null;
  const scrollElement = container || document.documentElement;
  const { top: offsetTop, left: offsetLeft } = getElementOffset(container);

  return {
    ...domRect,
    width: domRect.width || domRect.right - domRect.left,
    height: domRect.height || domRect.top - domRect.bottom,
    top: domRect.bottom + scrollElement.scrollTop - offsetTop,
    left: domRect.left + scrollElement.scrollLeft - offsetLeft,
    elementTop: domRect.top + scrollElement.scrollTop - offsetTop
  };
}

export function getRefRect(
  ref?: MutableRefObject<HTMLElement | null>,
  getContainer?: () => HTMLElement | null
): ReactiveDomReact {
  if (!ref || !ref.current) {
    return defaultRect;
  }
  const rect = ref.current.getBoundingClientRect();
  return getRectFromDOMWithContainer(rect, getContainer);
}
