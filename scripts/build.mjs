import fs, { existsSync } from "node:fs";
import { join, resolve } from "node:path";

import { logger } from "./logger.mjs";

// Display verbose output
const isVerbose =
  process.argv.includes("--verbose") || process.argv.includes("-v");

export const DEFAULT_PACKAGE_MANAGER = "pnpm";
const CWD = process.cwd();

// Build the compiler binary
const PKG_CORE = resolve(CWD, "./packages/core");

// Build cli
const PKG_CLI = resolve(CWD, "./packages/cli");

const PKG_RUNTIME = resolve(CWD, "./packages/runtime");

const PKG_RUNTIME_PLUGIN_HMR = resolve(CWD, "./packages/runtime-plugin-hmr");

const PKG_RUNTIME_PLUGIN_IMPORT_META = resolve(
  CWD,
  "./packages/runtime-plugin-import-meta",
);

// Build plugin-tools
const PKG_PLUGIN_TOOLS = resolve(CWD, "./packages/plugin-tools");

// Build plugin dts
const PKG_DTS = resolve(CWD, "./js-plugins/dts");

// Build ReplaceDirnamePlugin
const PKG_REPLACE_DIRNAME_PLUGIN = resolve(
  CWD,
  "./rust-plugins/replace-dirname",
);

// Build rust_plugin_react
const PKG_RUST_PLUGIN = resolve(CWD, "./rust-plugins");

// Build js_plugin_path
export const JS_PLUGINS_DIR = resolve(CWD, "./js-plugins");
export const EXAMPLES_DIR = resolve(CWD, "./examples");

export const excludedJsPlugin = ["dts"];

const execa = async (...args) => {
  const execaPkg = await import("execa");
  return execaPkg.execa(...args);
};

export const installDependencies = async () => {
  const { execSync } = await import("child_process");

  execSync(`${DEFAULT_PACKAGE_MANAGER} install`, {
    cwd: CWD,
    stdio: "inherit",
  });
};

export const executeStartProject = async () =>
  execa(DEFAULT_PACKAGE_MANAGER, ["start"], {
    cwd: CWD,
    stdio: "inherit",
  });

export const buildExamples = async ({ startFrom, example } = {}) => {
  const examples = fs.readdirSync("./examples").sort((a, b) =>
    a.localeCompare(b),
  );
  const builtRustPlugins = new Set();
  const hasPrebuiltRustPluginArtifact = (rustPluginPath) => {
    const npmPath = join(rustPluginPath, "npm");

    if (!existsSync(npmPath)) {
      return false;
    }

    for (const dirent of fs.readdirSync(npmPath, { withFileTypes: true })) {
      if (!dirent.isDirectory()) {
        continue;
      }

      if (existsSync(join(npmPath, dirent.name, "index.farm"))) {
        return true;
      }
    }

    return false;
  };

  const resolveRustPluginsForExample = (example, examplePath) => {
    const result = new Set();

    if (example.startsWith("rust-plugin-")) {
      result.add(example.slice("rust-plugin-".length));
    }

    const pkgJsonPath = join(examplePath, "package.json");
    if (!existsSync(pkgJsonPath)) {
      return [...result];
    }

    try {
      const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, "utf8"));
      const depNames = [
        ...Object.keys(pkg.dependencies || {}),
        ...Object.keys(pkg.devDependencies || {}),
      ];

      for (const depName of depNames) {
        if (!depName.startsWith("@farmfe/plugin-")) {
          continue;
        }

        const pluginName = depName.slice("@farmfe/plugin-".length);
        const pluginPath = resolve(PKG_RUST_PLUGIN, pluginName);

        if (existsSync(join(pluginPath, "package.json"))) {
          result.add(pluginName);
        }
      }
    } catch (error) {
      console.warn(`Failed to parse ${pkgJsonPath}: ${error}`);
    }

    return [...result];
  };

  const examplesToBuild = example
    ? (() => {
        const exampleIndex = examples.indexOf(example);

        if (exampleIndex === -1) {
          throw new Error(`Example '${example}' was not found under ./examples`);
        }

        return [examples[exampleIndex]];
      })()
    : startFrom
      ? (() => {
        const startIndex = examples.indexOf(startFrom);

        if (startIndex === -1) {
          throw new Error(
            `Example '${startFrom}' was not found under ./examples`,
          );
        }

        return examples.slice(startIndex);
        })()
      : examples;
  console.log("Building", examplesToBuild.length, "examples...");

  for (const example of examplesToBuild) {
    const examplePath = join("./examples", example);
    const rustPluginNames = resolveRustPluginsForExample(example, examplePath);

    for (const rustPluginName of rustPluginNames) {
      if (builtRustPlugins.has(rustPluginName)) {
        continue;
      }

      const rustPluginPath = resolve(PKG_RUST_PLUGIN, rustPluginName);

      if (!existsSync(join(rustPluginPath, "package.json"))) {
        continue;
      }

      console.log(
        `Building rust plugin ${rustPluginName} for example ${examplePath}`,
      );

      if (process.env.CI && hasPrebuiltRustPluginArtifact(rustPluginPath)) {
        console.log(
          `Skipping rust plugin ${rustPluginName} build in CI because prebuilt artifact is available`,
        );
        builtRustPlugins.add(rustPluginName);
        continue;
      }

      await execa("npm", ["run", "build"], {
        cwd: rustPluginPath,
        stdio: isVerbose ? "inherit" : "ignore",
      });

      builtRustPlugins.add(rustPluginName);
    }

    if (!existsSync(join(examplePath, "package.json"))) {
      continue;
    }
    console.log("Building", examplePath);

    if (fs.statSync(examplePath).isDirectory()) {
      await execa("npm", ["run", "build"], {
        cwd: examplePath,
      });
    }
  }
};

export async function runTaskQueue() {
  await runTask("Cli", buildCli);
  await runTask("Runtime", buildRuntime);
  await runTask("PluginTools", buildPluginTools);
  await runTask("Core", buildCore);
  await runTask("RustPlugins", buildRustPlugins);
  await runTask("JsPlugins", buildJsPlugins);
}

// build core command
export const buildCore = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ["build:rs"], {
    cwd: PKG_CORE,
    stdio: isVerbose ? "inherit" : "ignore",
  })
    .then(buildReplaceDirnamePlugin)
    .then(buildCoreCjs);

export const buildCoreCjs = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ["build:cjs"], {
    cwd: PKG_CORE,
  });

// build cli command
export const buildCli = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ["build"], {
    cwd: PKG_CLI,
  });

export const buildRuntime = async () => {
  await execa(DEFAULT_PACKAGE_MANAGER, ["build"], {
    cwd: PKG_RUNTIME,
  });
  return Promise.all([
    execa(DEFAULT_PACKAGE_MANAGER, ["build"], {
      cwd: PKG_RUNTIME_PLUGIN_HMR,
    }),
    execa(DEFAULT_PACKAGE_MANAGER, ["build"], {
      cwd: PKG_RUNTIME_PLUGIN_IMPORT_META,
    }),
  ]);
};

// build farm-plugin-tools
export const buildPluginTools = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ["build"], {
    cwd: PKG_PLUGIN_TOOLS,
  });

// build dts command
export const buildDts = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ["build"], {
    cwd: PKG_DTS,
  });

export const buildReplaceDirnamePlugin = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ["build"], {
    cwd: PKG_REPLACE_DIRNAME_PLUGIN,
  });

// build rust plugins
export const rustPlugins = () => batchBuildPlugins(PKG_RUST_PLUGIN);

export const buildJsPlugins = async (spinner) => {
  const jsPluginDirs = fs.readdirSync(JS_PLUGINS_DIR).filter((file) => {
    return (
      fs.statSync(join(JS_PLUGINS_DIR, file)).isDirectory() &&
      !excludedJsPlugin.includes(file)
    );
  });

  const total = jsPluginDirs.length;
  console.log("\n");
  logger(`Found ${total} JS plugins to build \n`, {
    color: "yellow",
    title: "Javascript Info",
  });
  await buildDts();
  for (const pluginDir of jsPluginDirs) {
    const pluginPath = resolve(JS_PLUGINS_DIR, pluginDir);
    await runTask(
      `Js plugin: ${pluginDir}`,
      async (spinner) => {
        try {
          if (!existsSync(join(pluginPath, "package.json"))) {
            spinner.warn({
              text: `Skipping ${pluginDir}: No package.json found`,
            });
            return;
          }
          await execa(DEFAULT_PACKAGE_MANAGER, ["build"], {
            cwd: pluginPath,
            stdio: isVerbose ? "inherit" : "ignore",
          });

          spinner.success({
            text: `📦 JS plugin \x1b[32m${pluginDir}\x1b[0m built successfully.`,
          });
        } catch (error) {
          spinner.error({ text: `Failed to build JS plugin: ${pluginDir}` });
          throw error;
        }
      },
      "Building",
      "Build",
      spinner,
      false,
    );
  }
};

export const buildRustPlugins = async (spinner) => {
  const filterPlugins = ["replace-dirname"];

  const rustPluginDirs = fs.readdirSync(PKG_RUST_PLUGIN).filter((file) => {
    return fs.statSync(join(PKG_RUST_PLUGIN, file)).isDirectory();
  });

  const buildPlugins = rustPluginDirs.filter(
    (item) => !filterPlugins.includes(item),
  );

  const total = buildPlugins.length;
  console.log("\n");
  logger(`Found ${total} Rust plugins to build \n`, {
    color: "rust",
    title: "Rust Info",
  });
  for (const pluginDir of buildPlugins) {
    const pluginPath = resolve(PKG_RUST_PLUGIN, pluginDir);
    await runTask(
      `Rust plugin: ${pluginDir}`,
      async (spinner) => {
        try {
          if (!existsSync(join(pluginPath, "Cargo.toml"))) {
            spinner.warn({
              text: `Skipping ${pluginDir}: No Cargo.toml found`,
            });
            return;
          }

          await execa("npm", ["run", "build"], {
            cwd: pluginPath,
            stdio: isVerbose ? "inherit" : "ignore",
          });

          spinner.success({
            text: `📦 Rust plugin \x1b[32m${pluginDir}\x1b[0m compiled successfully.`,
          });
        } catch (error) {
          spinner.error({ text: `Failed to build Rust plugin: ${pluginDir}` });
          throw error;
        }
      },
      "Building",
      "Build",
      spinner,
      false,
    );
  }
};

export async function runTask(
  taskName,
  task,
  processText = "Building",
  finishedText = "Build",
  spinner = null,
  showSuccess = true,
) {
  try {
    const { createSpinner } = await import("nanospinner");
    spinner = createSpinner();
  } catch (e) {
    // ignore error
  }
  try {
    await task(spinner?.start({ text: `${processText} ${taskName}` }));
    showSuccess
      ? spinner?.success({
          text: `✨ ✨ ${finishedText} ${taskName} completed! `,
        })
      : spinner?.reset();
  } catch (e) {
    spinner?.error({ text: `${finishedText} ${taskName} failed!` });
    console.error(e.toString());
    process.exit(1);
  }
}

export function resolveNodeVersion() {
  const currentVersion = process.versions.node;
  const requiredMajorVersion = parseInt(currentVersion.split(".")[0], 10);
  const minimumMajorVersion = 16;

  if (requiredMajorVersion < minimumMajorVersion) {
    logger(`Farm does not support using Node.js v${currentVersion}!`);
    logger(`Please use Node.js v${minimumMajorVersion} or higher.`);
    process.exit(1);
  }
}

export function batchBuildPlugins(
  baseDir,
  command = "build",
  packageManager = "pnpm",
) {
  const pluginNameMap = fs.readdirSync(baseDir).filter((file) => {
    return (
      fs.statSync(join(baseDir, file)).isDirectory() &&
      !excludedJsPlugin.includes(file)
    );
  });
  const path = pluginNameMap.map((subDir) => resolve(baseDir, subDir));
  return path.map((item) => {
    return execa(packageManager, [command], { cwd: item });
  });
}

export async function cleanBundleCommand() {
  try {
    await execa(DEFAULT_PACKAGE_MANAGER, [
      "-r",
      "--filter=./packages/*",
      "--filter=./js-plugins/*",
      "run",
      "clean",
    ]);
    console.log("");
    logger("pnpm clean command completed successfully.");
  } catch (error) {
    logger("An error occurred while running pnpm clean command:", {
      title: error.message,
      color: "red",
    });
    process.exit(1);
  }
}
