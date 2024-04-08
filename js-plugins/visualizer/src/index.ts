import { JsPlugin, UserConfig } from '@farmfe/core';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { RecordViewerOptions } from './types';
import { createRecordViewerServer } from './node/server';
import { createDateSourceMiddleware } from './node/dataSource';

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
