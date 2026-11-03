import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export function releaseNativeScope({ owners = '', profile, plugins, names }) {
  assert.ok(Array.isArray(plugins) && plugins.every(name => /^[a-z0-9-]+$/.test(name)));
  if (!owners) return { core: true, create: true, plugins };
  assert.equal(profile, 'release', 'CI cannot consume release native selection');
  const selected = JSON.parse(owners);
  assert.ok(Array.isArray(selected) && selected.length <= 64 && new Set(selected).size === selected.length);
  const known = new Set(['@farmfe/core', 'create-farm', ...Object.values(names)]);
  assert.ok(selected.every(name => typeof name === 'string' && known.has(name)), 'Unknown release native owner');
  return { core: selected.includes('@farmfe/core'), create: selected.includes('create-farm'),
    plugins: plugins.filter(directory => selected.includes(names[directory])) };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = fileURLToPath(new URL('../', import.meta.url)), names = {};
  for (const directory of fs.readdirSync(path.join(root, 'rust-plugins'), { withFileTypes: true })) {
    if (!directory.isDirectory() || directory.isSymbolicLink()) continue;
    const file = path.join(root, 'rust-plugins', directory.name, 'package.json');
    if (fs.existsSync(file)) names[directory.name] = JSON.parse(fs.readFileSync(file, 'utf8')).name;
  }
  const scope = releaseNativeScope({ owners: process.env.FARM_RELEASE_NATIVE_OWNERS, profile: process.env.FARM_BUILD_PROFILE,
    plugins: (process.argv[3] ?? '').split(' ').filter(Boolean), names });
  assert.ok(['core', 'create', 'plugins'].includes(process.argv[2]));
  console.log(process.argv[2] === 'plugins' ? scope.plugins.join(' ') : String(scope[process.argv[2]]));
}
