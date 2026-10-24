import { defineConfig } from '@farmfe/core';
import { common } from './configs/common';
import { rustPlugins } from './configs/rust';
export default defineConfig(() => ({ ...common('rust'), ...rustPlugins() }));
