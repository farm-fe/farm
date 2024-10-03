import { buildCli, buildCoreCjs, runTaskQueue } from './build.mjs';

await runTaskQueue();
await buildCli();
await buildCoreCjs();
