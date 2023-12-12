import { JsPlugin, UserConfig } from '@farmfe/core';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { RecordViewerOptions } from './types';
import { createRecordViewerServer } from './node/server';
import { createDateSourceMiddleware } from './node/dataSource';

const path = require('path');

const PLUGIN_DIR = path.dirname(fileURLToPath(import.meta.url));

export const PLUGIN_DIR_CLIENT = resolve(PLUGIN_DIR, '../client');

export default function farmRecorderPlugin(
  options: RecordViewerOptions = {}
): JsPlugin {
  let farmConfig: UserConfig['compilation'];
  let recordViewerOptions: RecordViewerOptions;

  return {
    name: 'farm-plugin-record-viewer',
    configResolved: (config) => {
      farmConfig = config || {};
      farmConfig.record = true;
      recordViewerOptions = options;
    },
    configureDevServer: (devServer) => {
      const compiler = devServer.getCompiler();

      const middleware = createDateSourceMiddleware(compiler);

      createRecordViewerServer({
        host: recordViewerOptions.host,
        port: recordViewerOptions.port,
        clientPath: PLUGIN_DIR_CLIENT,
        middleware
      });
    },
    configureCompiler: (compiler) => {
      if (farmConfig?.mode !== 'development') {
        const middleware = createDateSourceMiddleware(compiler);

        createRecordViewerServer({
          host: recordViewerOptions.host,
          port: recordViewerOptions.port,
          clientPath: PLUGIN_DIR_CLIENT,
          middleware
        });
      }
    }
  };
}
