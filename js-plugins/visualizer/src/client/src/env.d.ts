declare module '*.vue' {
  const component: DefineComponent<object, object, any>;
  export default component;
}
declare module '*.svg';
declare module '*.css';

declare module '*.worker?worker' {
  const worker: typeof Worker & {
    new (): Worker;
  };
  export default worker;
}

declare module 'monaco-editor/esm/vs/basic-languages/javascript/javascript' {
  export const conf: any;
  export const language: any;
}

declare module 'monaco-editor/esm/vs/basic-languages/typescript/typescript' {
  export const conf: any;
  export const language: any;
}