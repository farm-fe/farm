import { runTaskQueue, resolveNodeVersion } from './build.mjs';

resolveNodeVersion();
await runTaskQueue();
