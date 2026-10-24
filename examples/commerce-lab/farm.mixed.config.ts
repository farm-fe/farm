import { defineConfig } from '@farmfe/core';
import { common } from './configs/common';
import { mixedPlugins } from './configs/mixed';
export default defineConfig(() => ({ ...common('mixed'), ...mixedPlugins() }));
