import {build} from '@farmfe/core';
import {configurations} from './config.mjs';
for (const config of configurations(process.env.MF_MODE || 'production')) await build(config);
