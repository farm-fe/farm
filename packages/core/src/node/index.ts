export * from './compiler';
export * from './config';
export * from './server';

import { Compiler } from './compiler';
import { resolveUserConfig, UserConfig } from './config';
import { DevServer } from './server';

export async function start(options: { configPath?: string }): Promise<void> {
  const config: UserConfig = resolveUserConfig(options.configPath);
}
