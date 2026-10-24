import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import ts from 'typescript';
import { expect, it } from 'vitest';
// @ts-ignore Build helper is intentionally an executable JavaScript module.
import { createCjsTypes } from '../scripts/prepare-cjs-types.mjs';

it('bridges the reviewed generic server owner without erasing its instance type', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-owner-types-'));
  try {
    const entry = path.join(root, 'index.d.ts');
    await fs.writeFile(
      entry,
      'export interface OwnedRuntimeServer {close():Promise<void>} export declare class RuntimeServerOwner<T extends OwnedRuntimeServer>{readonly current:T|undefined;}'
    );
    const declaration = createCjsTypes(
      entry,
      ['RuntimeServerOwner'],
      ['RuntimeServerOwner']
    );
    await fs.writeFile(path.join(root, 'index.d.cts'), declaration);
    const consumer = path.join(root, 'consumer.cts');
    await fs.writeFile(
      consumer,
      `import {RuntimeServerOwner} from './index.cjs';type Server={close():Promise<void>;port:number};const owner:RuntimeServerOwner<Server>=new RuntimeServerOwner<Server>();const port:number|undefined=owner.current?.port;`
    );
    const program = ts.createProgram([consumer], {
      noEmit: true,
      strict: true,
      skipLibCheck: false,
      types: [],
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.NodeNext,
      moduleResolution: ts.ModuleResolutionKind.NodeNext
    });
    expect(
      ts
        .getPreEmitDiagnostics(program)
        .map((diagnostic) =>
          ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n')
        )
    ).toEqual([]);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});
