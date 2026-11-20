import {start} from '@farmfe/core';
import {configurations} from './config.mjs';
for (const config of configurations('development')) await start(config);
