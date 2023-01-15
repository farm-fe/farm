export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';

import { Compiler } from './compiler/index.js';
import { resolveUserConfig, UserConfig } from './config/index.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';

export async function start(options: { configPath?: string }): Promise<void> {
  const userConfig: UserConfig = await resolveUserConfig(options.configPath);
  const compiler = new Compiler(userConfig);
  const devServer = new DevServer(compiler, {
    writeToDisk: true,
  });

  if (userConfig?.server?.hmr) {
    // undefined means to use the default hmr config
    const userHmrConfig =
      userConfig.server.hmr === true ? undefined : userConfig.server.hmr;
    const fileWatcher = new FileWatcher(userConfig.root, userHmrConfig);
    fileWatcher.watch(compiler, devServer);
  }

  devServer.listen();
}

export async function build(options: { configPath?: string }): Promise<void> {
  const userConfig: UserConfig = await resolveUserConfig(options.configPath);
  const compiler = new Compiler(userConfig);
  await compiler.compile();
  compiler.writeResourcesToDisk();
}
