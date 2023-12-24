interface ImportMetaEnv {
  [key: string]: any;
  BASE_URL: string;
  MODE: string;
  DEV: boolean;
  PROD: boolean;
  SSR: boolean;
}

interface ImportMeta {
  url: string;

  readonly hot?: import('./hot').ViteHotContext;

  readonly env: ImportMetaEnv;

  glob: import('./import-glob').ImportGlobFunction;
}
