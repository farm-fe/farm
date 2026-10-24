import assert from 'node:assert/strict';
import path from 'node:path';

export function browserOptions(env) {
  const configs = { js: 'farm.config.ts', rust: 'farm.rust.config.ts', mixed: 'farm.mixed.config.ts' };
  const groups = (env.COMMERCE_BROWSER_GROUPS ?? 'js,rust,mixed').split(',');
  assert.ok(groups.length && groups.every(group => Object.hasOwn(configs, group)) && new Set(groups).size === groups.length, 'Invalid browser groups');
  const override = env.COMMERCE_CONFIG_OVERRIDE;
  if (override !== undefined) assert.ok(path.isAbsolute(override), 'Config override must be absolute');
  const project = env.COMMERCE_NPM_PROJECT;
  if (project !== undefined) assert.ok(path.isAbsolute(project), 'Npm project must be absolute');
  const buildArgs = env.COMMERCE_NPM_CLI
    ? [env.COMMERCE_NPM_CLI, ...(project ? ['--prefix', project] : []), 'run', 'build']
    : [override ? '../../packages/cli/bin/farm.mjs' : 'scripts/run.mjs', 'build'];
  if (override) buildArgs.push(...(env.COMMERCE_NPM_CLI ? ['--'] : []), '--config', override);
  return { groups, buildArgs, configFor: group => override ?? configs[group] };
}
