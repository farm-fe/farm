import http from 'node:http';
import { Compiler } from '@farmfe/core';
import { getFarmEnvInfo } from './utils/envinfo';

export function createDateSourceMiddleware(compiler: Compiler) {
  return async (
    req: http.IncomingMessage,
    res: http.ServerResponse,
    next: () => Promise<any>
  ) => {
    const url = req.url as string;
    const { pathname, searchParams } = new URL(
      url,
      `http://${req.headers.host}`
    );

    if (pathname.startsWith('/__record')) {
      const id = searchParams.get('id') as string;
      const handleRecordRequest = (result: any) => {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(result));
      };

      if (pathname === '/__record/modules') {
        handleRecordRequest(compiler.modules());
      } else if (pathname === '/__record/resolve') {
        handleRecordRequest(compiler.getResolveRecords(id));
      } else if (pathname === '/__record/transform') {
        handleRecordRequest(compiler.getTransformRecords(id));
      } else if (pathname === '/__record/process') {
        handleRecordRequest(compiler.getProcessRecords(id));
      } else if (pathname === '/__record/analyze_deps') {
        handleRecordRequest(compiler.getAnalyzeDepsRecords(id));
      } else if (pathname === '/__record/resource_pot') {
        handleRecordRequest(compiler.getResourcePotRecordsById(id));
      } else if (pathname === '/__record/farm_env_info') {
        const info = await getFarmEnvInfo();
        if (typeof info === 'object') {
          handleRecordRequest(info);
        } else if (typeof info === 'string') {
          handleRecordRequest(JSON.parse(info));
        } else {
          handleRecordRequest(null);
        }
      } else if (pathname === '/__record/resources_map') {
        const resource_map = compiler.resourcesMap();
        handleRecordRequest(resource_map);
      } else if (pathname === '/__record/resource') {
        const resource = compiler.resource(id);
        handleRecordRequest(resource);
      } else if (pathname === '/__record/stats') {
        const stats = compiler.pluginStats();
        handleRecordRequest(stats);
      } else {
        await next();
      }
    } else {
      await next();
    }
  };
}
