export const injectSymbol = Symbol('_qiankun_helper_').toString();

export interface InjectOptions {
  bootstrap: () => Promise<void>;
  mount: (props: unknown) => Promise<void>;
  unmount: () => Promise<void>;
  update?: (props: unknown) => Promise<void>;
}

export function injectQiankun(options: InjectOptions): void {
  // @ts-ignore
  window[injectSymbol] = {
    ...options
  };
}
