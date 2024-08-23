import { JsPlugin } from '@farmfe/core';
export type LessPluginOptions = {
    lessOptions?: Less.Options;
    implementation?: string;
    filters?: {
        resolvedPaths?: string[];
        moduleTypes?: string[];
    };
    additionalData?: string | ((content?: string, resolvePath?: string) => string | Promise<string>);
};
export default function farmLessPlugin(options?: LessPluginOptions): JsPlugin;
