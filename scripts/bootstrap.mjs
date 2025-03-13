import {
  runTaskQueue,
  cleanBundleCommand,
  installDependencies,
  executeStartProject,
} from "./build.mjs";

const CI = process.env.CI || process.argv.includes("--ci");

await installDependencies();
await cleanBundleCommand();
await runTaskQueue();

!CI && (await executeStartProject());
