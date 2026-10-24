import { rustPlugins } from './rust';
import { data } from '../adapters/data';

/** Data stays on JS until the Rust export contract passes the capability suite. */
export function mixedPlugins() {
  const config = rustPlugins();
  return {
    plugins: [...config.plugins.slice(0, 5), data(), ...config.plugins.slice(8)],
  };
}
