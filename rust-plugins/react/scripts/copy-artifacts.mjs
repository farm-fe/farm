import { copyFile } from 'fs/promises';
import path from 'path';

const npmDirMap = {
  darwin_x64: 'darwin-x64',
  darwin_arm64: 'darwin-arm64',
  linux_x64: 'linux-x64-gnu',
  win32_x64: 'win32-x64-msvc',
  win32_ia32: 'win32-ia32-msvc',
};

let platformArch = `${process.platform}_${process.arch}`;

console.log(process.argv);

if (process.argv.includes('aarch64-apple-darwin')) {
  platformArch = 'darwin_arm64';
}

const copyArtifacts = async () => {
  await copyFile(`farm-plugin-react.${npmDirMap[platformArch]}.node`, path.join('npm', npmDirMap[platformArch], 'index.farm'));
  console.log(`Copied artifacts from farm-plugin-react.${npmDirMap[platformArch]}.node to npm/${npmDirMap[platformArch]}/index.farm`);
}

copyArtifacts();
