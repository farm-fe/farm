import { copyFile } from 'fs/promises';
import path from 'path';

export const copyArtifacts = async (abi?: string) => {
  if (!abi) {
    // try local abi
    const supportedAbis = [
      'darwin-arm64',
      'darwin-x64',
      'linux-arm64-gnu',
      'linux-arm64-musl',
      'linux-x64-gnu',
      'linux-x64-musl',
      'win32-x64-msvc'
    ];
    const localAbi = process.platform + '-' + process.arch;
    console.log('localAbi', localAbi);
    const found = supportedAbis.find((abi) => abi.includes(localAbi));
    if (found) {
      abi = found;
    } else {
      throw new Error('Missing --abi');
    }
  }

  // find .node file
  const files = await import('fs').then((m) => m.promises.readdir('.'));
  const nodeFile = files.find(
    (file) => file.endsWith('.node') && file.includes(abi)
  );

  if (!nodeFile) {
    console.log('files:\n', files);
    throw new Error(
      'Missing .node file in current directory: ' + process.cwd()
    );
  }

  await copyFile(nodeFile, path.join('npm', abi, 'index.farm'));
  console.log(`Copied artifacts from ${nodeFile} to npm/${abi}/index.farm`);
};
