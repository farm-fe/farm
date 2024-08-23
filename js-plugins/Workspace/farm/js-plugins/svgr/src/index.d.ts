import type { JsPlugin } from '@farmfe/core';
import type { Config as SvgrOptions } from '@svgr/core';
export interface FarmSvgrPluginOptions {
    svgrOptions?: SvgrOptions;
    filters?: {
        resolvedPaths?: string[];
    };
}
export default function farmSvgrPlugin(options?: FarmSvgrPluginOptions): JsPlugin;
