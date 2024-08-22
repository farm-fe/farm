import { JsPlugin } from '@farmfe/core';
import postcssLoadConfig from 'postcss-load-config';
import type { base } from './aa.js';
export type PostcssPluginOptions = {
    /**
     * @default undefined
     * postcss-load-config options. path default to farm.config.js root.
     */
    postcssLoadConfig?: {
        ctx?: postcssLoadConfig.ConfigContext;
        path?: string | base;
        options?: Parameters<typeof postcssLoadConfig>[2];
    };
    filters?: {
        resolvedPaths?: string[];
        moduleTypes?: string[];
    };
    implementation?: string;
    internalPlugins?: {
        /**
         * @default false
         * @description please see https://www.npmjs.com/package/postcss-import
         */
        postcssImport?: boolean;
    };
};
export default function farmPostcssPlugin(options?: PostcssPluginOptions): JsPlugin;
