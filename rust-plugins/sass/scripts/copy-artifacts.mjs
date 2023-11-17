import { copyFile } from 'fs/promises';
import path from 'path';

const [abiFlag, abi] = process.argv.slice(-2);
if (abiFlag !== '--abi') {
  throw new Error('Missing --abi');
}

const copyArtifacts = async () => {
  await copyFile(
    `farm-plugin-sass.${abi}.node`,
    path.join('npm', abi, 'index.farm')
  );
  console.log(
    `Copied artifacts from farm-plugin-sass.${abi}.node to npm/${abi}/index.farm`
  );
};

copyArtifacts();
