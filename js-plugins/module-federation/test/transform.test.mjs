import ts from 'typescript';
import test from 'node:test';
import assert from 'node:assert/strict';
import federation, {rewriteRemoteImports} from '../src/index.mjs';
test('rewrites only remote dynamic expressions, preserving comments, strings and local imports', () => {
  const source = `// import('remote/Fake')\nconst text = "import('remote/Fake')";\nconst a = import('remote/Button'); const b = import('./local.js');`;
  const result = rewriteRemoteImports(source, 'host.ts', ['remote']);
  assert.match(result.content, /__farm_mf_load__\("remote\/Button"\)/);
  assert.ok(result.content.includes("// import('remote/Fake')"));
  assert.ok(result.content.includes("import('./local.js')"));
  const map = JSON.parse(result.sourceMap);
  assert.equal(map.sourcesContent[0], source);
  assert.ok(map.mappings.length);
});
test('does not rewrite lookalikes and unrelated dynamic imports', () => {
  assert.equal(rewriteRemoteImports("import('remote-other/X'); import(path);", 'x.js', ['remote']), null);
});
test('rejects static imports, reexports and recognizable computed remotes', () => {
  for (const code of ["import x from 'remote/Button'", "export * from 'remote/Button'", 'import(`remote/${name}`)', "import('remote/' + name)"]) {
    assert.throws(() => rewriteRemoteImports(code, 'x.js', ['remote']), /not supported/);
  }
});
test('avoids introduced binding collisions', () => {
  const r = rewriteRemoteImports("const __farm_mf_load__ = 1; import('remote/Button')", 'x.js', ['remote']);
  assert.match(r.content, /as __farm_mf_load___/);
});
test('rejects unsupported configuration', () => {
  assert.throws(() => federation({name:'x',shared:{}}), /shared/);
  assert.throws(() => federation({name:'bad-name'}), /identifier/);
  assert.throws(() => federation({name:'x',filename:'../entry.js'}), /filename/);
  assert.throws(() => federation({name:'x'}).config({compilation:{runtime:{namespace:'other'}}}), /namespace/);
});

test('container lifecycle rejects uninitialized access, unknown exposes and sharing', async () => {
  const vm = await import('node:vm');
  const plugin = federation({name:'remoteTest', exposes:{'./Button':'./Button.js'}});
  plugin.config({root:'/probe',compilation:{output:{publicPath:'http://localhost:4101/deploy/'}}});
  const {resolvedPath} = plugin.resolve.executor({source:'farm-mf:container'});
  const {content} = plugin.load.executor({resolvedPath});
  const compiled = ts.transpileModule(content, {compilerOptions:{module:ts.ModuleKind.CommonJS}}).outputText;
  const paths = [];
  const context = vm.createContext({exports:{},farm_mf_remoteTest:{m:{sp:value => paths.push(...value)}}});
  vm.runInContext(compiled, context);
  assert.equal(paths[0], 'http://localhost:4101/deploy/');
  const container = context.remoteTest;
  await assert.rejects(container.get('./Button'), /init must run/);
  const scope = {};
  await container.init(scope);
  await container.init(scope);
  await assert.rejects(container.init({}), /Different share scope/);
  await assert.rejects(container.get('./Missing'), /Unknown expose: .\/Missing/);
  await assert.rejects(container.get('__proto__'), /Unknown expose/);
  assert.throws(() => vm.runInContext(`{${compiled}}`,context), /Duplicate container/);
});

test('runtime namespace derives from federation name rather than package name', () => {
  const config = federation({name:'remoteA'}).config({root:'/same',compilation:{}});
  assert.equal(config.compilation.runtime.namespace, 'farm_mf_remoteA');
  assert.equal(config.compilation.runtime.isolate, true);
});

test('accepts normalized virtual entry and dev base plus origin', () => {
  const plugin = federation({name:'devRemote',exposes:{'./Button':'./Button.js'}});
  plugin.config({root:'/probe',compilation:{output:{publicPath:'/deploy/'}},server:{origin:'http://localhost:4101'}});
  const direct = plugin.resolve.executor({source:'farm-mf:container'});
  assert.deepEqual(plugin.resolve.executor({source:'./farm-mf:container'}),direct);
  assert.match(plugin.load.executor(direct).content,/http:\/\/localhost:4101\/deploy\//);
});
test('rejects remote without deployment public path', () => {
  assert.throws(() => federation({name:'remote',exposes:{'./Button':'./Button.js'}}).config({root:'/probe'}),/publicPath/);
});
