import {spawn} from 'node:child_process';
import {once} from 'node:events';

if(!process.env.CODEX_DOCS_NATIVE_BASELINE)throw Error('[DOCS_NATIVE_BASELINE] guarded test run requires a frozen baseline');
await import('./farm-docs-native-guard.mjs');
const environment={...process.env};
delete environment.NODE_TEST_CONTEXT;
const child=spawn(process.execPath,['--import',new URL('./farm-docs-native-guard.mjs',import.meta.url).href,'--test','--test-concurrency=1',...process.argv.slice(2)],{stdio:'inherit',env:environment});
const [code]=await once(child,'close');
process.exitCode=typeof code==='number'?code:1;
