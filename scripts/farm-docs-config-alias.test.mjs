import assert from 'node:assert/strict';
import { withFrameworkAliases } from '../packages/docs/dist/resolve-alias.js';

const framework = { 'react$': 'framework-react', '@farmfe/docs/client$': 'framework-client' };
const user = [{ find: '@site', replacement: '/site' }, { find: /^local\//i, replacement: '/local/' }];
const ordered = withFrameworkAliases(user, framework);
assert.ok(Array.isArray(ordered));
assert.equal(ordered[2], user[0]);
assert.equal(ordered[3], user[1]);
assert.equal(ordered[3].find.flags, 'i');
assert.deepEqual(ordered.slice(0, 2), Object.entries(framework).map(([find, replacement]) => ({ find, replacement })));
assert.deepEqual(withFrameworkAliases({ '@site': '/site', 'react$': 'custom-react' }, framework), { '@site': '/site', ...framework });
assert.deepEqual(withFrameworkAliases(undefined, framework), framework);
assert.equal(Object.keys(ordered).some(key => ordered[key]?.find === undefined), false);
console.log('Framework alias regression passed: ordered entries, regex identity/flags, object configuration and reserved runtime precedence.');
