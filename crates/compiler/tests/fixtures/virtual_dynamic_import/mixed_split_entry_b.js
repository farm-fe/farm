import { shared } from './mixed_shared.js';
export const value = shared;
export const loadInline = () => import('./mixed_inline_entry_a.js');
