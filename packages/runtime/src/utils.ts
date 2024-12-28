import type { ModuleSystem } from "./module-system.js";
import type { FarmRuntimePluginContainer } from "./modules/plugin.js";

// Injected when runtime starts execution
declare const __farm_global_this__: any;

export function getModuleSystem(): ModuleSystem {
  return __farm_global_this__.m;
}

export function getPluginContainer(): FarmRuntimePluginContainer {
  return __farm_global_this__.p
}