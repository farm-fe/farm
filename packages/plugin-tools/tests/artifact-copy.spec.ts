import { execFileSync } from 'node:child_process';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { expect, it } from 'vitest';

const moduleUrl = new URL('../dist/copy-artifacts.js', import.meta.url).href;
const abi = 'win32-x64-msvc';

async function fixture(source: Buffer, destination?: Buffer) {
  const directory = await fs.mkdtemp(
    path.join(os.tmpdir(), 'farm-copy-artifact-')
  );
  const from = path.join(directory, `probe.${abi}.node`);
  const to = path.join(directory, 'npm', abi, 'index.farm');
  await fs.mkdir(path.dirname(to), { recursive: true });
  await fs.writeFile(from, source);
  if (destination) await fs.writeFile(to, destination);
  return { directory, from, to };
}

function copy(directory: string, setup = '') {
  // Exercise the normally built CLI helper in a fresh process. The payload is
  // a byte fixture, never loaded as a native module.
  return execFileSync(
    process.execPath,
    [
      '--input-type=module',
      '-e',
      `${setup}\nconst {copyArtifacts}=await import(${JSON.stringify(moduleUrl)});await copyArtifacts(${JSON.stringify(abi)});`
    ],
    { cwd: directory, windowsHide: true, encoding: 'utf8', timeout: 30000 }
  );
}

it('keeps an identical artifact untouched instead of overwriting it', async () => {
  const data = await fixture(
    Buffer.from('same artifact bytes'),
    Buffer.from('same artifact bytes')
  );
  const old = new Date('2000-01-01T00:00:00Z');
  await fs.utimes(data.to, old, old);
  const before = await fs.stat(data.to);
  copy(data.directory);
  expect(await fs.readFile(data.to)).toEqual(await fs.readFile(data.from));
  expect((await fs.stat(data.to)).mtimeMs).toBe(before.mtimeMs);
});

it('copies a missing artifact from the current build output', async () => {
  const data = await fixture(Buffer.from('new artifact'));
  copy(data.directory);
  expect(await fs.readFile(data.to)).toEqual(await fs.readFile(data.from));
});

it('replaces a different artifact even when lengths match', async () => {
  const data = await fixture(
    Buffer.from('new bytes'),
    Buffer.from('old bytes')
  );
  copy(data.directory);
  expect(await fs.readFile(data.to)).toEqual(Buffer.from('new bytes'));
});

it('compares the entire binary including a changed tail beyond a comparison chunk', async () => {
  const source = Buffer.alloc(192 * 1024, 37),
    destination = Buffer.from(source);
  destination[destination.length - 1] = 38;
  const data = await fixture(source, destination);
  copy(data.directory);
  expect(await fs.readFile(data.to)).toEqual(source);
});

it.each(['source', 'destination'])(
  'copies the current artifact when the %s path is replaced while its old handle stays open',
  async (side) => {
    const data = await fixture(
      Buffer.from('original'),
      Buffer.from('original')
    );
    const swapped = side === 'source' ? data.from : data.to;
    const marker = path.join(data.directory, 'replacement-observed');
    const setup = `
      const {createRequire,syncBuiltinESMExports}=await import('node:module');
      const require=createRequire(import.meta.url), fs=require('node:fs/promises'), path=require('node:path');
      const originalOpen=fs.open; let replaced=false;
      fs.open=async function(file,...args){
        const handle=await originalOpen.call(this,file,...args);
        if(!replaced && path.resolve(file)===${JSON.stringify(swapped)}){
          replaced=true; await fs.rename(file,file+'.retired');
          await fs.writeFile(file,'replaced'); await fs.writeFile(${JSON.stringify(marker)},'yes');
        }
        return handle;
      };
      syncBuiltinESMExports();`;
    copy(data.directory, setup);
    expect(await fs.readFile(marker, 'utf8')).toBe('yes');
    expect(await fs.readFile(data.to, 'utf8')).toBe(
      side === 'source' ? 'replaced' : 'original'
    );
  }
);
