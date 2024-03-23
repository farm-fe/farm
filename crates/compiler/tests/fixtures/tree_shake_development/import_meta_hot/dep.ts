if (import.meta.hot) {
  // accept dependencies
  import.meta.hot.accept(() => {
    import.meta.hot.invalidate('parent module should accept this');
  });
}

export const a = '1';
export const b = '2';
export function invalidate() {
  return `invalidate data`;
}
