import { type JsPlugin, type UserConfig } from '@farmfe/core';
export interface BuildOptions {
    /**
     * Shortcut of `compilation.input`
     */
    input: string | Record<string, string>;
    farm?: UserConfig;
}
export interface ElectronPluginOptions {
    main: BuildOptions;
    preload?: BuildOptions;
}
export default function farmElectronPlugin(options: ElectronPluginOptions): JsPlugin;
