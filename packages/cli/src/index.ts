import { cac } from "cac";
import { COMMANDS } from "./plugin/index.js";
import { filterDuplicateOptions, resolveCore } from "./utils.js";
import { performance } from "node:perf_hooks";
import { logger } from "./utils.js";
import { VERSION } from "./constants.js";
import path from "node:path";

const cli = cac("farm");

// common command
cli
  .option("-c, --config <file>", `use specified config file`)
  .option("-m, --mode <mode>", `set env mode`);

// dev command
cli
  .command(
    "",
    "Compile the project in dev mode and serve it with farm dev server"
  )
  .alias("start")
  .alias("dev")
  .option("--host [host]", "specify host")
  .option("--port [port]", "specify port")
  .option("--open", "open browser on server start")
  .option("--https", "use https")
  .option("--strictPort", "specified port is already in use, exit with error")
  .action(async (options) => {
    filterDuplicateOptions(options);
    const root = path.join(process.cwd(), options.config ?? "");
    // TODO add runtime command config path
    try {
      const { start } = await resolveCore(root);
      await start({
        configPath: root,
        ...options,
      });
    } catch (e) {
      // TODO refactor Error
      logger(e.message, { title: "Farm Error", color: "red" });
      process.exit(1);
    }
  });

// build command
cli
  .command("build", "Compile the project in production mode")
  .option("--target <target>", "transpile target")
  .option("--outDir <dir>", "output directory")
  // TODO sourcemap output config path
  .option("--sourcemap", "output source maps for build")
  .option("--minify", "code compression at build time")
  .option("-w, --watch", `rebuilds when files have changed on disk`)
  .action(async (options: any) => {
    const root = path.join(process.cwd(), options.config ?? "");
    filterDuplicateOptions(options);
    try {
      const { build } = await resolveCore(root);
      build({
        configPath: root,
        ...options,
      });
    } catch (e) {
      logger(e.message, { title: "Farm Error", color: "red" });
      process.exit(1);
    }
  });

// watch command
cli.command("watch", "rebuilds when files have changed on disk");

// create plugins command
cli
  .command("plugin [command]", "Commands for manage plugins", {
    allowUnknownOptions: true,
  })
  // TODO refactor plugin command
  .action((command: keyof typeof COMMANDS, args: unknown) => {
    COMMANDS[command](args);
  });

cli.help();

cli.version(VERSION);

cli.parse();
