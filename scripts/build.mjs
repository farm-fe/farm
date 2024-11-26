import fs, { existsSync } from "node:fs";
import os from "node:os";
import { join, resolve } from "node:path";
import { execa } from "execa";
import { createSpinner } from "nanospinner";

import { logger } from "./logger.mjs";

export const DEFAULT_PACKAGE_MANAGER = "pnpm";
const DEFAULT_HOMEBREW_PACKAGE_MANAGER = "brew";
const DEFAULT_LINUX_PACKAGE_MANAGER = "apt";
const CWD = process.cwd();

// Build the compiler binary
const PKG_CORE = resolve(CWD, "./packages/core");

// Build cli
const PKG_CLI = resolve(CWD, "./packages/cli");

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

const skipExamples = [
  // circle build when run `pnpm run build`
  "electron",
  // @swc/plugin-emotion version conflict
  "emotion",
  // solid js plugin alias compatible
  "solid",
  "solid-ssr",
  // tree shake failed when farm build
  "vanilla-extract",
];

export const buildExamples = async () => {
  const examples = fs.readdirSync("./examples");
  console.log("Building", examples.length, "examples...");

  for (const example of examples) {
    const examplePath = join("./examples", example);
    if (
      !existsSync(join(examplePath, "package.json")) ||
      skipExamples.includes(example)
    ) {
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
  // The sass plug-in uses protobuf, so you need to determine whether the user installs it or not.
  await installProtoBuf();
  await runTask("Cli", buildCli);
  await runTask("PluginTools", buildPluginTools);
  await runTask("Core", buildCore);
  await runTask("RustPlugins", buildRustPlugins);
  await runTask("JsPlugins", buildJsPlugins);
  await runTask("Artifacts", copyArtifacts);
}

// install mac protobuf
export const installMacProtobuf = () =>
  execa(DEFAULT_HOMEBREW_PACKAGE_MANAGER, ["install", "protobuf"], {
    cwd: CWD,
  });

// install linux protobuf
export const installLinuxProtobuf = async (spinner) => {
  if (isDebianSeries()) {
    try {
      await execa("type", DEFAULT_LINUX_PACKAGE_MANAGER);
      return execa(
        DEFAULT_LINUX_PACKAGE_MANAGER,
        ["install", "-y", "protobuf-compiler"],
        {
          cwd: CWD,
        },
      );
    } catch (_) {
      return Promise.reject(
        `not found "${DEFAULT_LINUX_PACKAGE_MANAGER}", if it's not your package manager, please install "protobuf" manually.`,
      );
    }
  } else if (isArchLinux()) {
    try {
      await execa("which", ["pacman"]);
      let result;
      if (process.getuid() == 0) {
        result = execa("pacman", ["-Sy", "protobuf"], {
          cwd: CWD,
          input: "y\n",
        });
      } else {
        spinner.stop();
        result = execa("sudo", ["pacman", "-Sy", "protobuf"], {
          cwd: CWD,
          stdin: "inherit",
          stdout: "inherit",
          stderr: "inherit",
        });
        spinner.start();
      }
      return result;
    } catch (err) {
      console.log(err);
      return Promise.reject(
        `not found "pacman", if it's not your package manager, please install "protobuf" manually.`,
      );
    }
  } else {
    return Promise.reject(``);
  }
};

// build core command
export const buildCore = () =>
  execa(DEFAULT_PACKAGE_MANAGER, ["build:rs"], {
    cwd: PKG_CORE,
    stdio: "inherit",
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
            // stdio: "pipe",
            stdio: "inherit",
          });

          spinner.success({ text: `📦 Built JS plugin: ${pluginDir} ` });
        } catch (error) {
          spinner.error({ text: `Failed to build JS plugin: ${pluginDir}` });
          throw error;
        }
      },
      "Building",
      "Build",
      spinner,
    );
  }
  spinner.start();
};

export const buildRustPlugins = async (spinner) => {
  const rustPluginDirs = fs.readdirSync(PKG_RUST_PLUGIN).filter((file) => {
    return (
      fs.statSync(join(PKG_RUST_PLUGIN, file)).isDirectory() &&
      !excludedJsPlugin.includes(file)
    );
  });
  const filterPlugins = ["replace-dirname"];
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
            // stdio: "pipe",
            stdio: "inherit",
          });

          spinner.success({ text: `📦 Built Rust plugin: ${pluginDir}` });
        } catch (error) {
          spinner.error({ text: `Failed to build Rust plugin: ${pluginDir}` });
          throw error;
        }
      },
      "Building",
      "Build",
      spinner,
    );
  }
};

export const copyArtifacts = (_) =>
  batchBuildPlugins(PKG_RUST_PLUGIN, "copy-artifacts");

export async function runTask(
  taskName,
  task,
  processText = "Building",
  finishedText = "Build",
  spinner = createSpinner(),
) {
  try {
    await task(spinner.start({ text: `${processText} ${taskName}` }));
    spinner.success({ text: `✨ ✨ ${finishedText} ${taskName} completed! ` });
  } catch (e) {
    spinner.error({ text: `${finishedText} ${taskName} failed!` });
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

export function isMac() {
  const platform = os.platform();
  return platform === "darwin";
}

export function isLinux() {
  const platform = os.platform();
  return platform === "linux";
}

export function isWindows() {
  const platform = os.platform();
  return platform === "win32";
}

export function getLinuxDistribution() {
  const data = fs.readFileSync("/etc/os-release", {
    encoding: "utf8",
  });

  const config = {};
  const lines = data.split("\n");

  lines.forEach((line) => {
    const trimmedLine = line.trim();
    if (trimmedLine && !trimmedLine.startsWith("#")) {
      const [key, value] = trimmedLine.split("=");
      const cleanKey = key.trim();
      const cleanValue = value.trim().replace(/"/g, "");

      config[cleanKey] = cleanValue;
    }
  });

  return config.ID;
}

export function isArchLinux() {
  return getLinuxDistribution() === "arch";
}

export function isDebianSeries() {
  const distro = getLinuxDistribution();
  return (
    distro === "debian" ||
    distro === "ubuntu" ||
    distro === "linuxmint" ||
    distro === "raspbian"
  );
}

export async function checkProtobuf() {
  const isWindowsFlag = isWindows();
  const isMacFlag = isMac();
  const isLinuxFlag = isLinux();
  try {
    if (isWindowsFlag) {
      await execa("where", ["protoc"]);
    } else if (isMacFlag || isLinuxFlag) {
      await execa("which", ["protoc"]);
    }
    return true;
  } catch {
    return false;
  }
}

export async function installProtoBuf() {
  const installFlag = await checkProtobuf();
  if (!installFlag) {
    logger(
      "Due to the use of protoc in the project, we currently judge that you have not installed. we need to install protobuf locally to make the project start successfully. \n\n- For mac users, will be use your local `homebrew` tool for installation. (First, Make sure your computer has `homebrew` installed) \n- For linux users, we will use your local `apt` tool for installation. (First, Make sure your computer has `apt` installed) \n- For Windows users, because the protobuf plugin cannot be installed automatically, You need to install manually according to the prompts \n",
      { title: "FARM WARN", color: "yellow" },
    );

    if (isMac()) {
      await runTask("Protobuf", installMacProtobuf, "Install", "Install");
    } else if (isLinux()) {
      await runTask("Protobuf", installLinuxProtobuf, "Install", "Install");
    }

    if (isWindows()) {
      logger(
        "If you are using a windows system, you can install it in the following ways:\n\n 1. open https://github.com/protocolbuffers/protobuf \n If you are a 32-bit operating system install https://github.com/protocolbuffers/protobuf/releases/download/v21.7/protoc-21.7-win32.zip \n If you are a 64-bit operating system install https://github.com/protocolbuffers/protobuf/releases/download/v21.7/protoc-21.7-win64.zip \n 2. After installation, find the path you installed, and copy the current path, adding to the environment variable of windows \n\n Or you can directly check out the following article to install \n https://www.geeksforgeeks.org/how-to-install-protocol-buffers-on-windows/",
        { title: "FARM TIPS", color: "yellow" },
      );
      process.exit(1);
    }
  } else {
    console.log("");
    logger("Protobuf has been installed, skipping installation. \n");
  }
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
    logger("pnpm clean command completed successfully.");
  } catch (error) {
    logger("An error occurred while running pnpm clean command:", {
      title: error.message,
      color: "red",
    });
    process.exit(1);
  }
}
