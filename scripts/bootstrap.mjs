import {
  cleanBundleCommand,
  executeStartProject,
  installDependencies,
  runTaskQueue,
} from "./build.mjs";

await installDependencies();
await cleanBundleCommand();
await runTaskQueue();
await executeStartProject();
