import { copyFile } from 'fs/promises';
import path from 'path';

let [abiFlag, abi] = process.argv.slice(-2);
if (abiFlag !== '--abi') {
  // try local abi
  const supportedAbis = ['darwin-arm64', 'darwin-x64', 'linux-arm64-gnu', 'linux-arm64-musl', 'linux-x64-gnu', 'win32-x64-msvc'];
  const localAbi = process.platform + '-' + process.arch;
  console.log('localAbi', localAbi);
  const found = supportedAbis.find(abi => abi.includes(localAbi));
  if (found) {
    abi = found;
  } else {
    throw new Error('Missing --abi');
  }
}

const copyArtifacts = async () => {
  await copyFile(
    `farm-plugin-react.${abi}.node`,
    path.join('npm', abi, 'index.farm')
  );
  console.log(
    `Copied artifacts from farm-plugin-react.${abi}.node to npm/${abi}/index.farm`
  );
};

copyArtifacts();
