import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { afterEach, describe, expect, test } from 'vitest';
import Context from '../src/context.js';

const roots: string[] = [];
function getProject(ctx: Context) {
  if (!ctx.project) throw new Error('Declaration project was not initialized');
  return ctx.project;
}
afterEach(() => {
  for (const root of roots.splice(0))
    fs.rmSync(root, { recursive: true, force: true });
});

function setup(
  compilerOptions: Record<string, unknown> = {},
  options: any = {}
) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-dts-issue-1783-'));
  roots.push(root);
  fs.writeFileSync(
    path.join(root, 'index.ts'),
    'export const answer: number = 42;'
  );
  fs.writeFileSync(
    path.join(root, 'tsconfig.json'),
    JSON.stringify({
      compilerOptions: { skipLibCheck: true, ...compilerOptions },
      include: ['*.ts', '*.js']
    })
  );
  const ctx = new Context();
  ctx.handleResolveOptions(options, {
    root,
    mode: 'production',
    output: { path: 'dist' }
  } as any);
  return { root, ctx };
}

describe('declaration compiler options (#1783)', () => {
  test('emits TypeScript declarations with isolatedDeclarations from tsconfig', () => {
    const { root, ctx } = setup({ isolatedDeclarations: true });
    const source = getProject(ctx).addSourceFileAtPath(
      path.join(root, 'index.ts')
    );
    expect(
      getProject(ctx)
        .getPreEmitDiagnostics()
        .map((d) => d.getMessageText())
    ).toEqual([]);
    const output = source.getEmitOutput(true);
    expect(output.getEmitSkipped()).toBe(false);
    expect(
      output
        .getOutputFiles()
        .map((f) => f.getText())
        .join('\n')
    ).toContain('export declare const answer: number;');
  });

  test('respects explicit allowJs false from tsconfig', () => {
    const { ctx } = setup({ allowJs: false });
    expect(getProject(ctx).getCompilerOptions().allowJs).toBe(false);
  });

  test('respects explicit compilerOptions allowJs false', () => {
    const { ctx } = setup({}, { compilerOptions: { allowJs: false } });
    expect(getProject(ctx).getCompilerOptions().allowJs).toBe(false);
  });

  test('supports isolatedDeclarations from plugin compilerOptions', () => {
    const { root, ctx } = setup(
      {},
      { compilerOptions: { isolatedDeclarations: true } }
    );
    const source = getProject(ctx).addSourceFileAtPath(
      path.join(root, 'index.ts')
    );
    expect(
      getProject(ctx)
        .getPreEmitDiagnostics()
        .map((d) => d.getMessageText())
    ).toEqual([]);
    expect(source.getEmitOutput(true).getEmitSkipped()).toBe(false);
  });

  test('supports isolatedDeclarations through an inherited tsconfig', () => {
    const { root } = setup();
    fs.writeFileSync(
      path.join(root, 'base.json'),
      JSON.stringify({
        compilerOptions: { isolatedDeclarations: true, skipLibCheck: true }
      })
    );
    fs.writeFileSync(
      path.join(root, 'tsconfig.json'),
      JSON.stringify({ extends: './base.json' })
    );
    const ctx = new Context();
    ctx.handleResolveOptions({}, { root, output: { path: 'dist' } } as any);
    getProject(ctx).addSourceFileAtPath(path.join(root, 'index.ts'));
    expect(
      getProject(ctx)
        .getPreEmitDiagnostics()
        .map((d) => d.getMessageText())
    ).toEqual([]);
  });

  test('continues to emit JavaScript declarations by default', () => {
    const { root, ctx } = setup();
    const source = getProject(ctx).createSourceFile(
      path.join(root, 'index.js'),
      'export const answer = 42;'
    );
    const output = source.getEmitOutput(true);
    expect(output.getEmitSkipped()).toBe(false);
    expect(
      output
        .getOutputFiles()
        .map((f) => f.getText())
        .join('\n')
    ).toContain('export const answer: 42;');
  });
});
