import type { JsPlugin } from '@farmfe/core';
import type { LegacyOptions, StringOptions } from 'sass';
export type SassPluginOptions<Legacy = boolean> = {
    sassOptions?: Partial<Legacy extends false ? StringOptions<'async'> : LegacyOptions<'async'>>;
    filters?: {
        resolvedPaths?: string[];
        moduleTypes?: string[];
    };
    /**
     * Use legacy sass API. E.g `sass.render` instead of `sass.compileStringAsync`.
     */
    legacy?: Legacy;
    /**
     * - relative or absolute path
     * - globals file will be added to the top of the sass file
     * - when file changed, the file can't be hot-reloaded
     *
     * relative to project root or cwd
     */
    implementation?: string | undefined;
    globals?: string[];
    additionalData?: string | ((content?: string, resolvePath?: string) => string | Promise<string>);
};
export default function farmSassPlugin(options?: SassPluginOptions): JsPlugin;
