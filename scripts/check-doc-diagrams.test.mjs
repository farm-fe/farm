import assert from 'node:assert/strict';import fs from 'node:fs/promises';import path from 'node:path';import os from 'node:os';import {fileURLToPath}from'node:url';import{test}from'node:test';import{checkDiagrams,validateSvg}from'./check-doc-diagrams.mjs';
const source=fileURLToPath(new URL('../website/static/diagrams/',import.meta.url));
test('committed SVGs match all maintained Mermaid sources',async()=>assert.deepEqual(await checkDiagrams(),{sources:6,outputs:12}));

test('complete Rust lifecycle retains all hooks and a separate update entry',async()=>{
 const names=['rust-plugin-lifecycle','rust-plugin-build','rust-plugin-generate','rust-plugin-update'];
 const text=(await Promise.all(names.map(name=>fs.readFile(path.join(source,name+'.mmd'),'utf8')))).join('\n');
 const hooks=['config','plugin_cache_loaded','build_start','resolve','load','transform','parse','process_module','analyze_deps','finalize_module','freeze_module','module_graph_build_end','build_end','generate_start','optimize_module_graph','freeze_module_graph_meta','analyze_module_graph','partial_bundling','process_resource_pots','render_start','render_resource_pot','process_rendered_resource_pot','augment_resource_pot_hash','optimize_resource_pot','generate_resources','process_generated_resources','handle_entry_resource','finalize_resources','generate_end','finish','update_modules','module_graph_updated','update_finished','handle_persistent_cached_module','write_plugin_cache'];
 assert.equal(new Set(hooks).size,35);
 for(const hook of hooks)assert.match(text,new RegExp('\\b'+hook+'\\b'),'Missing hook '+hook);
 assert.match(text,/module_graph_updated\[module_graph_updated\]:::serial/);
 const update=await fs.readFile(path.join(source,'rust-plugin-update.mmd'),'utf8');
 assert.match(update,/Watcher event \/ Compiler.update/);
 assert(!/finish\s*-->\s*update_modules/.test(text),'Update must not be an unconditional continuation');
});
test('source and rendered SVG tampering fail rather than using stale diagrams',async()=>{const temp=await fs.mkdtemp(path.join(os.tmpdir(),'farm-diagram-test-'));try{await fs.cp(source,temp,{recursive:true});await fs.appendFile(path.join(temp,'plugin-dispatch.mmd'),'\nchanged');await assert.rejects(checkDiagrams(temp),/Stale Mermaid source/);await fs.copyFile(path.join(source,'plugin-dispatch.mmd'),path.join(temp,'plugin-dispatch.mmd'));await fs.appendFile(path.join(temp,'plugin-dispatch-light.svg'),'changed');await assert.rejects(checkDiagrams(temp),/Stale SVG/);}finally{await fs.rm(temp,{recursive:true,force:true});}});
test('SVGs cannot carry scripts, HTML, events, external references or stylesheets',()=>{for(const svg of ['<svg><script/></svg>','<svg><foreignObject/></svg>','<svg onload="run()"/>','<svg><use href="https://example.com/x"/></svg>','<svg><style>rect{fill:url(https://example.com/x)}</style></svg>'])assert.throws(()=>validateSvg(svg));validateSvg('<svg><path marker-end="url(#arrow)"/></svg>');});
