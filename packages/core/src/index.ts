export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';

import { Compiler } from './compiler/index.js';
import {
  normalizeUserCompilationConfig,
  resolveUserConfig,
  UserConfig,
} from './config/index.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';

export async function start(options: { configPath?: string }): Promise<void> {
  const userConfig: UserConfig = await resolveUserConfig(options.configPath);
  const normalizedConfig = await normalizeUserCompilationConfig(userConfig);
  const compiler = new Compiler(normalizedConfig);
  const devServer = new DevServer(compiler, userConfig.server);

  if (devServer.config.hmr) {
    const fileWatcher = new FileWatcher(userConfig.root, devServer.config.hmr);
    fileWatcher.watch(devServer);
  }

  devServer.listen();
}

export async function build(options: { configPath?: string }): Promise<void> {
  const userConfig: UserConfig = await resolveUserConfig(options.configPath);
  const normalizedConfig = await normalizeUserCompilationConfig(userConfig);
  const compiler = new Compiler(normalizedConfig);
  await compiler.compile();
  compiler.writeResourcesToDisk();
}
