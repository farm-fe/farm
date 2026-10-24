import { defineConfig } from '@farmfe/core';
import { common } from './configs/common';
import { jsPlugins } from './configs/js';
export default defineConfig(() => ({ ...common('js'), ...jsPlugins() }));
