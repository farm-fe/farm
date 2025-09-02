import { execSync } from 'child_process';
import { buildCorePkg, buildCoreCjs, buildJsPlugins, buildRuntime } from './build.mjs';

// build node packages
await buildCorePkg();
await buildRuntime();
await buildCoreCjs();
await buildJsPlugins();

execSync('npm config set access public', { stdio: 'inherit' });
// publish node packages
execSync('npx changeset publish', { stdio: 'inherit' });
