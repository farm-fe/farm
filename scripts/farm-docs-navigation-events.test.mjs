import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import vm from 'node:vm';
import { test } from 'node:test';
const require = createRequire(import.meta.url);
const ts = require(process.env.FARM_DOCS_TEST_TYPESCRIPT || 'typescript');
const source = fs.readFileSync(new URL('../packages/docs-theme-default/src/navigation.tsx', import.meta.url), 'utf8');
const compiled = ts.transpileModule(source, { compilerOptions: { jsx: ts.JsxEmit.ReactJSX, target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } });
const document = { activeElement: null };
const module = { exports: {} };
vm.runInNewContext(compiled.outputText, { exports: module.exports, document, require: name => {
 if (name !== 'react/jsx-runtime') throw new Error('Unexpected runtime dependency ' + name);
 const jsx = (type, props) => ({ type, props }); return { jsx, jsxs: jsx };
} });
function fixture(hover = true) {
 const menu = module.exports.NavigationMenu({ label: 'Section', name: 'Section', children: null, hover });
 const summary = { focus() { document.activeElement = summary; } };
 const links = [0, 1].map(index => ({ index, focus() { document.activeElement = this; } }));
 const element = { open: false, contains: target => target === summary || links.includes(target), querySelectorAll: () => links, querySelector: () => summary };
 return { handlers: menu.props, element, summary, links };
}
test('mouse hover opens, touch keeps native click behavior and focused children retain menu', () => {
 const { handlers, element, links } = fixture(); document.activeElement = null;
 handlers.onPointerEnter({pointerType:'touch', currentTarget:element}); assert.equal(element.open,false);
 handlers.onPointerEnter({pointerType:'mouse', currentTarget:element}); assert.equal(element.open,true);
 document.activeElement=links[0]; handlers.onPointerLeave({pointerType:'mouse',currentTarget:element}); assert.equal(element.open,true);
 document.activeElement=null; handlers.onPointerLeave({pointerType:'mouse',currentTarget:element}); assert.equal(element.open,false);
});
test('arrow navigation wraps and Escape closes and restores summary focus', () => {
 const { handlers, element, summary, links } = fixture(); document.activeElement=summary;
 const key = value => handlers.onKeyDown({key:value,currentTarget:element,preventDefault(){},stopPropagation(){}});
 key('ArrowDown'); assert.equal(element.open,true); assert.equal(document.activeElement,links[0]);
 key('ArrowUp'); assert.equal(document.activeElement,links[1]);
 key('Escape'); assert.equal(element.open,false); assert.equal(document.activeElement,summary);
});
test('blur outside and actual link selection dismiss without swallowing native navigation', () => {
 const { handlers, element } = fixture(); element.open=true;
 handlers.onBlur({currentTarget:element,relatedTarget:null}); assert.equal(element.open,false);
 element.open=true; handlers.onClick({currentTarget:element,target:{closest:()=>({})}}); assert.equal(element.open,false);
});

test('hover opt-out retains click and keyboard group semantics', () => {
 const { handlers, element, links } = fixture(false); document.activeElement = null;
 handlers.onPointerEnter({pointerType:'mouse',currentTarget:element}); assert.equal(element.open,false);
 handlers.onKeyDown({key:'ArrowDown',currentTarget:element,preventDefault(){},stopPropagation(){}});
 assert.equal(element.open,true); assert.equal(document.activeElement,links[0]);
});
test('pointer movement into the popup does not close a hover selector', () => {
 const { handlers, element, links } = fixture(true); document.activeElement = null; element.open=true;
 handlers.onPointerLeave({pointerType:'mouse',currentTarget:element,relatedTarget:links[0]});
 assert.equal(element.open,true);
});
