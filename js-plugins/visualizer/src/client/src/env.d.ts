declare module '*.vue' {
  const component: DefineComponent<object, object, any>;
  export default component;
}
declare module '*.svg';

declare module '*.worker?worker' {
  const worker: typeof Worker & {
    new (): Worker;
  };
  export default worker;
}