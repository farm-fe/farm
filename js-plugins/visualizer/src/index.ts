import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Compiler, JsPlugin, UserConfig } from '@farmfe/core';
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
  let compiler: Compiler, host: string, port: number;

  return {
    name: 'farm-visualizer',
    config(config) {
      farmConfig = config.compilation || {};
      farmConfig.record = true;
      return config;
    },
    configureCompiler: (c) => {
      compiler = c;
      const middleware = createDateSourceMiddleware(compiler);

      let { host: h, port: p } = createRecordViewerServer({
        host: recordViewerOptions.host,
        port: recordViewerOptions.port,
        clientPath: PLUGIN_DIR_CLIENT,
        middleware
      });
      host = h;
      port = p;
    },
    finish: {
      async executor() {
        console.log(
          `[finish] Farm Record Viewer run at http://${host}:${port}`
        );
      }
    },
    updateFinished: {
      executor() {
        // set message to client to refresh stats
      }
    }
  };
}
