import { existsSync, readFileSync } from 'fs';
import { createRequire } from 'module';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const { platform, arch } = process;
const currentDir = dirname(fileURLToPath(import.meta.url));

let binPath = null;

const require = createRequire(import.meta.url);

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      return readFileSync('/usr/bin/ldd', 'utf8').includes('musl');
    } catch (e) {
      return true;
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header;
    return !glibcVersionRuntime;
  }
}

switch (platform) {
  case 'win32':
    switch (arch) {
      case 'x64':
        if (existsSync(join(currentDir, '../npm/win32-x64-msvc/index.farm'))) {
          binPath = join(currentDir, '../npm/win32-x64-msvc/index.farm');
        } else {
          binPath = require.resolve('@farmfe/module-federation-win32-x64-msvc');
        }

        break;
      case 'ia32':
        if (existsSync(join(currentDir, '../npm/win32-ia32-msvc/index.farm'))) {
          binPath = join(currentDir, '../npm/win32-ia32-msvc/index.farm');
        } else {
          binPath = require.resolve('@farmfe/module-federation-win32-ia32-msvc');
        }

        break;
      case 'arm64':
        if (existsSync(join(currentDir, '../npm/win32-arm64-msvc/index.farm'))) {
          binPath = join(currentDir, '../npm/win32-arm64-msvc/index.farm');
        } else {
          binPath = require.resolve('@farmfe/module-federation-win32-arm64-msvc');
        }

        break;
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`);
    }
    break;
  case 'darwin':
    switch (arch) {
      case 'x64':
        if (existsSync(join(currentDir, '../npm/darwin-x64/index.farm'))) {
          binPath = join(currentDir, '../npm/darwin-x64/index.farm');
        } else {
          binPath = require.resolve('@farmfe/module-federation-darwin-x64');
        }
        break;
      case 'arm64':
        if (existsSync(join(currentDir, '../npm/darwin-arm64/index.farm'))) {
          binPath = join(currentDir, '../npm/darwin-arm64/index.farm');
        } else {
          binPath = require.resolve('@farmfe/module-federation-darwin-arm64');
        }
        break;
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`);
    }
    break;
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          if (existsSync(join(currentDir, '../npm/linux-x64-musl/index.farm'))) {
            binPath = join(currentDir, '../npm/linux-x64-musl/index.farm');
          } else {
            binPath = require.resolve('@farmfe/module-federation-linux-x64-musl');
          }
        } else {
          if (existsSync(join(currentDir, '../npm/linux-x64-gnu/index.farm'))) {
            binPath = join(currentDir, '../npm/linux-x64-gnu/index.farm');
          } else {
            binPath = require.resolve('@farmfe/module-federation-linux-x64-gnu');
          }
        }

        break;

      case 'arm64':
        if (isMusl()) {
          if (
            existsSync(join(currentDir, '../npm/linux-arm64-musl/index.farm'))
          ) {
            binPath = join(currentDir, '../npm/linux-arm64-musl/index.farm');
          } else {
            binPath = require.resolve('@farmfe/module-federation-linux-arm64-musl');
          }
        } else {
          if (
            existsSync(join(currentDir, '../npm/linux-arm64-gnu/index.farm'))
          ) {
            binPath = join(currentDir, '../npm/linux-arm64-gnu/index.farm');
          } else {
            binPath = require.resolve('@farmfe/module-federation-linux-arm64-gnu');
          }
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`);
    }
    break;
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

export default binPath;
