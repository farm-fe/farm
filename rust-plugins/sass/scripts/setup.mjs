import * as fs from 'node:fs/promises';
import * as p from 'node:path';
import { spawnSync } from 'node:child_process';
import fetch from 'node-fetch';
import extractZip from 'extract-zip';
import { extract as extractTar } from 'tar';

const OS = (() => {
  const platform = process.platform;
  switch (platform) {
    case 'linux':
      return 'linux';
    case 'darwin':
      return 'macos';
    case 'win32':
      return 'windows';
    default:
      throw Error(`Platform ${platform} is not supported.`);
  }
})();

const ARCH = (() => {
  const arch = process.arch;
  switch (arch) {
    case 'ia32':
      return 'ia32';
    case 'x86':
      return 'ia32';
    case 'x64':
      return 'x64';
    case 'arm64':
      return 'arm64';
    default:
      throw Error(`Architecure ${arch} is not supported.`);
  }
})();

const ARCHIVE_EXTENSION = OS === 'windows' ? '.zip' : '.tar.gz';

async function cleanDir(dir) {
  await fs.mkdir(p.dirname(dir), { recursive: true });
  try {
    await fs.rm(dir, { recursive: true });
  } catch (_) {
    // If dir doesn't exist yet, that's fine.
  }
}

async function download(url) {
  console.log(`Downloading "${url}".`);
  const response = await fetch(url, { redirect: 'follow' });
  if (!response.ok) {
    throw Error(`Failed to download ${url}: ${response.statusText}`);
  }
  return Buffer.from(await response.arrayBuffer());
}

async function getDartSassEmbedded(outPath, dirName) {
  const version = JSON.parse(
    await fs.readFile(p.join('./ext/sass/package.json'), 'utf8')
  ).dependencies['sass-embedded'];

  const releaseAsset = await download(
    `https://github.com/sass/dart-sass-embedded/releases/download/${version}/sass_embedded-${version}-${OS}-${ARCH}${ARCHIVE_EXTENSION}`
  );

  console.log(`Unzipping dart-sass-embedded release asset to ${outPath}.`);
  await cleanDir(p.join(outPath, dirName));
  const zippedAssetPath = `${outPath}/${dirName}${ARCHIVE_EXTENSION}`;
  await fs.writeFile(zippedAssetPath, Buffer.from(releaseAsset));
  if (OS === 'windows') {
    await extractZip(zippedAssetPath, {
      dir: p.join(process.cwd(), outPath),
    });
  } else {
    extractTar({
      file: zippedAssetPath,
      cwd: outPath,
      sync: true,
    });
  }
  await fs.unlink(zippedAssetPath);
  await fs.rename(p.join(outPath, 'sass_embedded'), p.join(outPath, dirName));
}

async function getEmbeddedProtocol(outPath, dirName) {
  const { stdout } = spawnSync(
    p.join(
      `./ext/sass/${dirName}/dart-sass-embedded${
        OS === 'windows' ? '.bat' : ''
      }`
    ),
    ['--version']
  );
  const { protocolVersion } = JSON.parse(stdout.toString());
  const assetUrl = `https://github.com/sass/embedded-protocol/raw/${protocolVersion}/embedded_sass.proto`;
  const asset = await download(assetUrl);
  await fs.writeFile(p.join(outPath, `${dirName}.proto`), Buffer.from(asset));
}

(async () => {
  try {
    await getDartSassEmbedded('./ext/sass', 'sass-embedded');
    await getEmbeddedProtocol('./ext/sass', 'sass-embedded');
  } catch (error) {
    console.error(error);
    process.exitCode = 1;
  }
})();
