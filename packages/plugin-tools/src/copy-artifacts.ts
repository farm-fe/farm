import { copyFile, open, stat } from 'fs/promises';
import path from 'path';

async function sameArtifact(source: string, destination: string) {
  const sourceHandle = await open(source, 'r');
  let destinationHandle: Awaited<ReturnType<typeof open>> | undefined;
  try {
    try {
      destinationHandle = await open(destination, 'r');
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') return false;
      throw error;
    }
    const [sourceBefore, destinationBefore] = await Promise.all([
      sourceHandle.stat(),
      destinationHandle.stat()
    ]);
    if (sourceBefore.size !== destinationBefore.size) return false;
    const sourceBuffer = Buffer.alloc(64 * 1024);
    const destinationBuffer = Buffer.alloc(sourceBuffer.length);
    for (
      let offset = 0;
      offset < sourceBefore.size;
      offset += sourceBuffer.length
    ) {
      const length = Math.min(sourceBuffer.length, sourceBefore.size - offset);
      const [sourceRead, destinationRead] = await Promise.all([
        sourceHandle.read(sourceBuffer, 0, length, offset),
        destinationHandle.read(destinationBuffer, 0, length, offset)
      ]);
      if (
        sourceRead.bytesRead !== length ||
        destinationRead.bytesRead !== length ||
        !sourceBuffer
          .subarray(0, length)
          .equals(destinationBuffer.subarray(0, length))
      )
        return false;
    }
    const [sourceAfter, destinationAfter] = await Promise.all([
      sourceHandle.stat(),
      destinationHandle.stat()
    ]);
    let identities;
    try {
      identities = await Promise.all([
        stat(source, { bigint: true }),
        stat(destination, { bigint: true }),
        sourceHandle.stat({ bigint: true }),
        destinationHandle.stat({ bigint: true })
      ]);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') return false;
      throw error;
    }
    const [sourcePath, destinationPath, sourceIdentity, destinationIdentity] =
      identities;
    return (
      sourceAfter.size === sourceBefore.size &&
      sourceAfter.mtimeMs === sourceBefore.mtimeMs &&
      sourceAfter.ctimeMs === sourceBefore.ctimeMs &&
      destinationAfter.size === destinationBefore.size &&
      destinationAfter.mtimeMs === destinationBefore.mtimeMs &&
      destinationAfter.ctimeMs === destinationBefore.ctimeMs &&
      sourcePath.dev === sourceIdentity.dev &&
      sourcePath.ino === sourceIdentity.ino &&
      sourcePath.size === sourceIdentity.size &&
      sourcePath.mtimeNs === sourceIdentity.mtimeNs &&
      sourcePath.ctimeNs === sourceIdentity.ctimeNs &&
      destinationPath.dev === destinationIdentity.dev &&
      destinationPath.ino === destinationIdentity.ino &&
      destinationPath.size === destinationIdentity.size &&
      destinationPath.mtimeNs === destinationIdentity.mtimeNs &&
      destinationPath.ctimeNs === destinationIdentity.ctimeNs
    );
  } finally {
    await Promise.all([sourceHandle.close(), destinationHandle?.close()]);
  }
}

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

  const destination = path.join('npm', abi, 'index.farm');
  if (await sameArtifact(nodeFile, destination)) {
    console.log(`Artifacts already match: ${nodeFile} and ${destination}`);
    return;
  }
  await copyFile(nodeFile, destination);
  console.log(`Copied artifacts from ${nodeFile} to npm/${abi}/index.farm`);
};
