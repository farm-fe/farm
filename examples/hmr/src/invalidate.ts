import { data } from "./data";

export function invalidate() {
  return `invalidate: ${data()}`;
}

if (import.meta.hot) {
  // accept dependencies
  import.meta.hot.accept(() => {
    import.meta.hot.invalidate('parent module should accept this');
  });
}