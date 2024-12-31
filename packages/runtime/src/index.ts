/** 
 * This module only exports runtime types and does not export any runtime code.
 * The runtime code is injected in need during compile time. See crates/plugin_runtime and crates/plugin_library
 */
export type { Module, ModuleSystem } from './module-system.js';
export type { Resource } from './modules/dynamic-import.js';