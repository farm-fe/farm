import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { JsPlugin, UserConfig } from '@farmfe/core';
import { createDateSourceMiddleware } from './node/dataSource';
import { createRecordViewerServer } from './node/server';
import { RecordViewerOptions } from './types';

const PLUGIN_DIR = dirname(fileURLToPath(import.meta.url));

export const PLUGIN_DIR_CLIENT = resolve(PLUGIN_DIR, '../client');

export default function farmRecorderPlugin(
  options: RecordViewerOptions = {}
): JsPlugin {
  let farmConfig: UserConfig['compilation'];
  const recordViewerOptions: RecordViewerOptions = options;

  return {
    name: 'farm-visualizer',
    config(config) {
      farmConfig = config.compilation || {};
      farmConfig.record = true;
      return config;
    },
    configureCompiler: (compiler) => {
      const middleware = createDateSourceMiddleware(compiler);

      createRecordViewerServer({
        host: recordViewerOptions.host,
        port: recordViewerOptions.port,
        clientPath: PLUGIN_DIR_CLIENT,
        middleware
      });
    }
  };
}
