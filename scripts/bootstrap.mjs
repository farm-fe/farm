import {
  runTaskQueue,
  cleanBundleCommand,
  installDependencies,
  executeStartProject,
} from "./build.mjs";

await installDependencies();
await cleanBundleCommand();
await runTaskQueue();
await executeStartProject();
