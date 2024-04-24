declare const __DEV__: boolean;
declare module '*.svg' {
  const src: string;
  export default src;
}
declare module '*.png' {
  const src: string;
  export default src;
}
declare module '*.module.css' {
  const styles: Record<string, string>;
  export default styles;
}
declare module '*.module.sass' {
  const styles: Record<string, string>;
  export default styles;
}
declare module '*.module.scss' {
  const styles: Record<string, string>;
  export default styles;
}
declare module '*.module.less' {
  const styles: Record<string, string>;
  export default styles;
}
