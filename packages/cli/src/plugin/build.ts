import { execSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';

/**
 * Farm plugin build command, build a rust farm plugin
 */
export function build(_args: any[]): void {
  // TODO, should automatically install Rust environment automatically
  execSync(`cargo build --release`, {
    stdio: 'inherit',
  });
  const cwd = process.cwd();
  const cargoTomlPath = path.join(cwd, 'Cargo.toml');
  const cargoMetadata = JSON.parse(
    execSync(
      `cargo metadata --format-version 1 --manifest-path "${cargoTomlPath}"`,
      {
        stdio: 'pipe',
        maxBuffer: 1024 * 1024 * 10,
      }
    ).toString('utf8')
  );
  const packages = cargoMetadata.packages;
  const rootPackage = packages.find(
    (p: { id: string }) => p.id === cargoMetadata.resolve.root
  );
  const cargoArtifactName = rootPackage.name.replace(/-/g, '_');

  const platformMap: Record<
    string,
    { libExt: string; cargoArtifactName: string }
  > = {
    darwin: {
      libExt: '.dylib',
      cargoArtifactName: `lib${cargoArtifactName}`,
    },
    win32: {
      libExt: '.dll',
      cargoArtifactName,
    },
    linux: {
      libExt: '.so',
      cargoArtifactName: `lib${cargoArtifactName}`,
    },
  };

  const targetDir = findTargetDir(cwd);

  if (!platformMap[process.platform]) {
    throw new Error(`Unsupported platform: ${process.platform}`);
  }

  const npmDirMap: Record<string, string> = {
    darwin_x64: 'darwin-x64',
    darwin_arm64: 'darwin-arm64',
    linux_x64: 'linux-x64-gnu',
    win32_x64: 'win32-x64-msvc',
  };
  const { libExt, cargoArtifactName: artifactName } =
    platformMap[process.platform];
  const sourcePath = path.join(
    targetDir,
    'release',
    `${artifactName}${libExt}`
  );
  const platformArch = `${process.platform}_${process.arch}`;

  if (!npmDirMap[platformArch]) {
    throw new Error(`Unsupported platform: ${platformArch}`);
  }

  const destPath = path.join(cwd, 'npm', npmDirMap[platformArch], 'index.farm');
  execSync(`cp "${sourcePath}" "${destPath}"`);
}

function findTargetDir(cwd: string): string {
  const target = path.join(cwd, 'target');

  if (existsSync(target)) {
    return target;
  }

  const parent = path.dirname(cwd);
  return findTargetDir(parent);
}
