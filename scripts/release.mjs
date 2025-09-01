import { execSync } from 'child_process';
import { buildCli, buildCoreCjs, buildJsPlugins, buildRuntime } from './build.mjs';

// build node packages
await buildCli();
await buildRuntime();
await buildCoreCjs();
await buildJsPlugins();

execSync('npm config set access public', { stdio: 'inherit' });
// publish node packages
execSync('npx changeset publish --tag beta', { stdio: 'inherit' });
