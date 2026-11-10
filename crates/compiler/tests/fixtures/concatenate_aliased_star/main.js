export * from './m2.js';
import('./m3.js').then(d => console.log(d.c));
