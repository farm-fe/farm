import { cac } from "cac";
import { COMMANDS } from "./plugin/index.js";
import { cleanOptions, filterDuplicateOptions, resolveCore } from "./utils.js";
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
  //TODO add host config
  .option("--port [port]", "specify port")
  .option("--open", "open browser on server start")
  .option("--https", "use https")
  // TODO add strictPort open config
  .option("--hmr", "enable hot module replacement")
  .option("--strictPort", "specified port is already in use, exit with error")
  .action(async (options) => {
    filterDuplicateOptions(options);
    const root = path.join(process.cwd(), options.config ?? "");
    options.configPath = root;
    // TODO add runtime command config path
    try {
      const { start } = await resolveCore(root);
      await start(cleanOptions(options));
    } catch (e) {
      // TODO refactor logger
      logger(e.message, { title: "Farm Error", color: "red" });
      process.exit(1);
    }
  });

// build command
cli
  .command("build", "compile the project in production mode")
  .option("--target <target>", "transpile target")
  .option("--outDir <dir>", "output directory")
  // TODO sourcemap output config path
  .option("--sourcemap", "output source maps for build")
  .option("--minify", "code compression at build time")
  .action(async (options: any) => {
    filterDuplicateOptions(options);
    const root = path.join(process.cwd(), options.config ?? "");

    options.configPath = root;
    try {
      const { build } = await resolveCore(root);

      build(cleanOptions(options));
    } catch (e) {
      logger(e.message, { title: "Farm Error", color: "red" });
      process.exit(1);
    }
  });

// TODO add watch command
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
