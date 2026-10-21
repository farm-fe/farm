import { defineDocsConfig } from '@farmfe/docs';
import site from './site.json';
export default defineDocsConfig(site, {
  compilation: { output: { path: 'build' }, persistentCache: false, sourcemap: false }
});
