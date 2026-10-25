import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import { test } from 'node:test';
const require = createRequire(new URL('../package.json', import.meta.url));
const ts = require('typescript');
const React = require('react');
// Execute actual component handlers with a small hook scheduler. This verifies
// component semantics, not browser focus/clipboard permissions or layout.
function fixture(file, aliases = {}) {
  const states = [], refs = [], dependencies = [], effects = [];
  let cursor = 0;
  const hooks = {
    ...React,
    useState(initial) { const i = cursor++; if (!(i in states)) states[i] = initial; return [states[i], v => { states[i] = v; }]; },
    useRef(initial) { const i = cursor++; return refs[i] ??= { current: initial }; },
    useId() { cursor++; return 'controls'; },
    useEffect(work, deps) { const i = cursor++; if (!dependencies[i] || deps.some((d, n) => d !== dependencies[i][n])) { dependencies[i] = deps; effects.push(work); } }
  };
  const source = ts.transpileModule(fs.readFileSync(new URL(`../src/${file}`, import.meta.url), 'utf8'), {
    fileName: file, compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX }
  }).outputText;
  const module = { exports: {} };
  new Function('require', 'module', 'exports', source)(name => {
    if (Object.hasOwn(aliases, name)) return aliases[name];
    if (name === 'react') return hooks;
    if (name === './client.js') return { useSite: () => ({ basePath: '/' }) };
    if (name === './code-block-metadata.js') return { CODE_BLOCK_METADATA_MARKER: 'owned-marker' };
    return require(name);
  }, module, module.exports);
  return { exports: module.exports, render(name, props) { cursor = 0; return module.exports[name](props); }, mount() { effects.splice(0).forEach(work => work()); } };
}
function all(node, predicate, found = []) {
  if (React.isValidElement(node)) { if (predicate(node)) found.push(node); React.Children.forEach(node.props.children, child => all(child, predicate, found)); }
  return found;
}
test('tabs click, arrow/Home/End selection, disabled skipping and local horizontal reveal', () => {
  const oldDocument = globalThis.document;
  const list = { scrollLeft: 0, getBoundingClientRect: () => ({ left: 0, right: 100 }) };
  const focused = [];
  globalThis.document = { getElementById: id => ({ parentElement: list, focus: options => focused.push({ id, options }), getBoundingClientRect: () => ({ left: 120, right: 180 }) }) };
  try {
    const f = fixture('content-tabs.tsx');
    const props = { children: ['a', 'b', 'c'].map(value => React.createElement(f.exports.TabItem, { value, label: value, disabled: value === 'b', key: value }, value)) };
    f.render('Tabs', props); f.mount();
    let tree = f.render('Tabs', props);
    const tabs = () => all(tree, n => n.props.role === 'tab');
    function key(name, from) { let prevented = false; tabs().find(n => n.props.children === from).props.onKeyDown({ key: name, preventDefault() { prevented = true; } }); tree = f.render('Tabs', props); return prevented; }
    assert.equal(all(tree, n => n.props.role === 'tablist')[0].props['aria-orientation'], 'horizontal');
    assert.equal(key('ArrowRight', 'a'), true);
    assert.equal(tabs()[2].props['aria-selected'], true);
    assert.equal(list.scrollLeft, 80);
    assert.deepEqual(focused[0], { id: 'controls-tab-2', options: { preventScroll: true } });
    key('ArrowRight', 'c'); assert.equal(tabs()[0].props['aria-selected'], true);
    key('ArrowLeft', 'a'); assert.equal(tabs()[2].props['aria-selected'], true);
    key('Home', 'c'); assert.equal(tabs()[0].props['aria-selected'], true);
    key('End', 'a'); assert.equal(tabs()[2].props['aria-selected'], true);
    assert.equal(key('Tab', 'c'), false, 'Tab leaves the widget without a trap');
    tabs()[0].props.onClick({ preventDefault() {} }); tree = f.render('Tabs', props);
    assert.equal(tabs()[0].props.tabIndex, 0); assert.equal(tabs()[1].props.tabIndex, -1);
    assert.equal(all(tree, n => n.props.role === 'tabpanel').filter(n => !n.props.hidden).length, 1);
  } finally { globalThis.document = oldDocument; }
});
test('CommandTabs retains each compiled tool raw string in its own selectable CodeBlock', () => {
  const tabs = fixture('content-tabs.tsx');
  const code = fixture('code-block.tsx');
  const command = fixture('snippets-runtime.tsx', {
    './content-tabs.js': tabs.exports,
    './code-block.js': code.exports
  });
  const raw = ['npm install x\r\n', 'pnpm add x\n'];
  const element = command.render('CommandTabs', { children: ['npm', 'pnpm'].map((tool, index) => React.createElement('pre', {
    key: tool, 'data-command-tool': tool, 'data-code': raw[index]
  }, React.createElement('code', null, 'Rust-rendered tokens'))) });
  assert.equal(element.type, tabs.exports.Tabs);
  const items = React.Children.toArray(element.props.children);
  assert.deepEqual(items.map(item => item.props.value), ['npm', 'pnpm']);
  assert.deepEqual(items.map(item => item.props.children.props['data-code']), raw);
  assert.ok(items.every(item => item.props.children.type === code.exports.CodeBlock));
});
test('icon copy writes exact current raw text including CR/NUL and exposes feedback', async () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
  const writes = [];
  Object.defineProperty(globalThis, 'navigator', { configurable: true, value: { clipboard: { writeText: async text => writes.push(text) } } });
  try {
    const f = fixture('code-block.tsx');
    let props = { 'data-code': 'pnpm add x\r\n\0' };
    f.render('CodeBlock', props); f.mount();
    let tree = f.render('CodeBlock', props);
    let button = all(tree, n => n.type === 'button')[0];
    assert.equal(button.props['aria-label'], 'Copy code');
    assert.equal(all(button, n => n.type === 'svg').length, 1);
    assert.equal(all(button, n => n.type === 'span').length, 0);
    await button.props.onClick(); tree = f.render('CodeBlock', props);
    assert.deepEqual(writes, ['pnpm add x\r\n\0']);
    assert.equal(all(tree, n => n.type === 'button')[0].props['data-copy-state'], 'copied');
    props = { 'data-code': 'npm install x\n' };
    f.render('CodeBlock', props); f.mount(); tree = f.render('CodeBlock', props);
    await all(tree, n => n.type === 'button')[0].props.onClick();
    assert.deepEqual(writes, ['pnpm add x\r\n\0', 'npm install x\n']);
  } finally { if (descriptor) Object.defineProperty(globalThis, 'navigator', descriptor); else delete globalThis.navigator; }
});
