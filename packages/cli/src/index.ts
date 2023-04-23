import { cac } from "cac";
import { COMMANDS } from "./plugin/index.js";
import { filterDuplicateOptions, resolveCore } from "./utils.js";
import { performance } from "node:perf_hooks";
import { logger } from "./utils.js";

const cli = cac("farm");

// help options

// TODO add runtime command like mode, debug, filter logs
cli
  .option("-c, --config <file>", `use specified config file`)
  .option("-m, --mode <mode>", `set env mode`);

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
    const cwd = process.cwd();
    filterDuplicateOptions(options);
    // TODO add runtime command config path
    try {
      const { start } = await resolveCore(cwd);
      const res = performance.now();
      await start({
        configPath: cwd,
        ...options,
      });
      console.log(Math.ceil(performance.now() - res) + "ms");
    } catch (e) {
      // TODO refactor Error
      logger(e.message, { title: "Farm Error", color: "red" });
    }
  });

cli
  .command("build", "Compile the project in production mode")
  .option("--target <target>", "transpile target")
  .option("--outDir <dir>", "output directory")
  // TODO sourcemap output config path
  .option("--sourcemap", "output source maps for build")
  .option("--minify", "code compression at build time")
  .option("-w, --watch", `rebuilds when files have changed on disk`)
  .action(async (options: any) => {
    const cwd = process.cwd();
    filterDuplicateOptions(options);
    try {
      const { build } = await resolveCore(cwd);
      build({
        configPath: cwd,
        ...options,
      });
    } catch (e) {
      logger(e.message, { title: "Farm Error", color: "red" });
    }
  });

cli.command("help").action(() => {
  cli.outputHelp();
});

const pluginCmd = cli
  .command("plugin [command]", "Commands for manage plugins", {
    allowUnknownOptions: true,
  })
  // TODO refactor plugin command
  .action((command: keyof typeof COMMANDS, args: unknown) => {
    COMMANDS[command](args);
  });

pluginCmd.cli.help();

cli.help();

try {
  cli.parse();
} catch (e) {
  console.log(e);
  process.exit(1);
}
