import sync from './sync.cjs';
import cycle from './cycle-a.cjs';
import {value} from './async.js';
globalThis.runAsync=true;globalThis.runFailure=false;
const consumer=await import('./consumer.cjs');
const caught=await import('./caught.cjs');
const iife=await import('./iife.cjs');
window.result={sync:sync.value,cycle,esm:value,consumer:consumer,caught:caught,iife:iife,order:globalThis.order};
