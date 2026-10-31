import { existsSync } from 'node:fs';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
const require = createRequire(import.meta.url);
const { platform, arch } = process;
const suffix = platform === 'win32' ? 'msvc' : platform === 'linux'
  ? (process.report.getReport().header.glibcVersionRuntime ? 'gnu' : 'musl') : '';
if (!['win32', 'linux', 'darwin'].includes(platform)) {
  throw new Error(`Farm Docs snapshot plugin does not support ${platform}/${arch}`);
}
const abi = [platform, arch, suffix].filter(Boolean).join('-');
const local = fileURLToPath(new URL(`../npm/${abi}/index.farm`, import.meta.url));
export default existsSync(local) ? local : require.resolve(`@farmfe/plugin-docs-snapshot-${abi}`);
