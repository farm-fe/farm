import { JsUserConfig } from '../../../binding';

type FilteredUserConfigKeys = Exclude<
  keyof JsUserConfig,
  'jsPlugins' | 'wasmPlugins'
>;
type FilteredUserConfig = {
  [key in FilteredUserConfigKeys]: JsUserConfig[key];
};

export interface UserConfig extends FilteredUserConfig {
  plugins: string[];
}

export function normalizeUserConfig(config: UserConfig): JsUserConfig {
  const normalizedConfig: JsUserConfig = {
    input: config.input,
    wasmPlugins: [],
    jsPlugins: [],
  };

  return normalizedConfig;
}
