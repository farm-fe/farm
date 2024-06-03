const defaultDocument = globalThis.isClient ? window.document : undefined;
const defaultWindow = globalThis.isClient ? window : undefined;
const F = {};

function useFullscreen(target, options = {}) {
  const {
    document = defaultDocument,
    autoExit = false
  } = options;

  return document;
}

const { document: { document1 } = defaultWindow.document } = F;

export {
  useFullscreen,
  document1
}