const { existsSync, readFileSync } = require("fs");
const { join } = require("path");

const { platform, arch } = process;

let nativeBindingPath = null;
let localFileExisted = false;
let loadError = null;

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== "function") {
    try {
      return readFileSync("/usr/bin/ldd", "utf8").includes("musl");
    } catch (e) {
      return true;
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header;
    return !glibcVersionRuntime;
  }
}

switch (platform) {
  case "android":
    switch (arch) {
      case "arm64":
        localFileExisted = existsSync(
          join(__dirname, "farm.android-arm64.node"),
        );
        try {
          if (localFileExisted) {
            nativeBindingPath = require.resolve("./farm.android-arm64.node");
          } else {
            nativeBindingPath = require.resolve("farm-android-arm64");
          }
        } catch (e) {
          loadError = e;
        }
        break;
      case "arm":
        localFileExisted = existsSync(
          join(__dirname, "farm.android-arm-eabi.node"),
        );
        try {
          if (localFileExisted) {
            nativeBindingPath = require.resolve("./farm.android-arm-eabi.node");
          } else {
            nativeBindingPath = require.resolve("farm-android-arm-eabi");
          }
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Android ${arch}`);
    }
    break;
  case "win32":
    switch (arch) {
      case "x64":
        localFileExisted = existsSync(
          join(__dirname, "farm.win32-x64-msvc.node"),
        );
        try {
          if (localFileExisted) {
            nativeBindingPath = require.resolve("./farm.win32-x64-msvc.node");
          } else {
            nativeBindingPath = require.resolve("farm-win32-x64-msvc");
          }
        } catch (e) {
          loadError = e;
        }
        break;
      case "ia32":
        localFileExisted = existsSync(
          join(__dirname, "farm.win32-ia32-msvc.node"),
        );
        try {
          if (localFileExisted) {
            nativeBindingPath = require.resolve("./farm.win32-ia32-msvc.node");
          } else {
            nativeBindingPath = require.resolve("farm-win32-ia32-msvc");
          }
        } catch (e) {
          loadError = e;
        }
        break;
      case "arm64":
        localFileExisted = existsSync(
          join(__dirname, "farm.win32-arm64-msvc.node"),
        );
        try {
          if (localFileExisted) {
            nativeBindingPath = require.resolve("./farm.win32-arm64-msvc.node");
          } else {
            nativeBindingPath = require.resolve("farm-win32-arm64-msvc");
          }
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`);
    }
    break;
  case "darwin":
    switch (arch) {
      case "x64":
        localFileExisted = existsSync(join(__dirname, "farm.darwin-x64.node"));
        try {
          if (localFileExisted) {
            nativeBindingPath = require.resolve("./farm.darwin-x64.node");
          } else {
            nativeBindingPath = require.resolve("farm-darwin-x64");
          }
        } catch (e) {
          loadError = e;
        }
        break;
      case "arm64":
        localFileExisted = existsSync(
          join(__dirname, "farm.darwin-arm64.node"),
        );
        try {
          if (localFileExisted) {
            nativeBindingPath = require.resolve("./farm.darwin-arm64.node");
          } else {
            nativeBindingPath = require.resolve("farm-darwin-arm64");
          }
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`);
    }
    break;
  case "freebsd":
    if (arch !== "x64") {
      throw new Error(`Unsupported architecture on FreeBSD: ${arch}`);
    }
    localFileExisted = existsSync(join(__dirname, "farm.freebsd-x64.node"));
    try {
      if (localFileExisted) {
        nativeBindingPath = require.resolve("./farm.freebsd-x64.node");
      } else {
        nativeBindingPath = require.resolve("farm-freebsd-x64");
      }
    } catch (e) {
      loadError = e;
    }
    break;
  case "linux":
    switch (arch) {
      case "x64":
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, "farm.linux-x64-musl.node"),
          );
          try {
            if (localFileExisted) {
              nativeBindingPath = require.resolve("./farm.linux-x64-musl.node");
            } else {
              nativeBindingPath = require.resolve("farm-linux-x64-musl");
            }
          } catch (e) {
            loadError = e;
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, "farm.linux-x64-gnu.node"),
          );
          try {
            if (localFileExisted) {
              nativeBindingPath = require.resolve("./farm.linux-x64-gnu.node");
            } else {
              nativeBindingPath = require.resolve("farm-linux-x64-gnu");
            }
          } catch (e) {
            loadError = e;
          }
        }
        break;
      case "arm64":
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, "farm.linux-arm64-musl.node"),
          );
          try {
            if (localFileExisted) {
              nativeBindingPath = require.resolve(
                "./farm.linux-arm64-musl.node",
              );
            } else {
              nativeBindingPath = require.resolve("farm-linux-arm64-musl");
            }
          } catch (e) {
            loadError = e;
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, "farm.linux-arm64-gnu.node"),
          );
          try {
            if (localFileExisted) {
              nativeBindingPath = require.resolve(
                "./farm.linux-arm64-gnu.node",
              );
            } else {
              nativeBindingPath = require.resolve("farm-linux-arm64-gnu");
            }
          } catch (e) {
            loadError = e;
          }
        }
        break;
      case "arm":
        localFileExisted = existsSync(
          join(__dirname, "farm.linux-arm-gnueabihf.node"),
        );
        try {
          if (localFileExisted) {
            nativeBindingPath = require.resolve(
              "./farm.linux-arm-gnueabihf.node",
            );
          } else {
            nativeBindingPath = require.resolve("farm-linux-arm-gnueabihf");
          }
        } catch (e) {
          loadError = e;
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`);
    }
    break;
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

if (loadError) {
  throw loadError;
}

module.exports = nativeBindingPath;
