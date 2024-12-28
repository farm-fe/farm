import type { ModuleInitialization } from "../module-system.js";
import { getModuleSystem } from "../utils.js";

function updateModule(moduleId: string, init: ModuleInitialization): void {
  const modules = getModuleSystem().m();
  modules[moduleId] = init;
  clearCache(moduleId);
}

function deleteModule(moduleId: string): boolean {
  const modules = getModuleSystem().m();
  if (modules[moduleId]) {
    clearCache(moduleId);
    delete modules[moduleId];
    return true;
  } else {
    return false;
  }
}

function clearCache(moduleId: string): boolean {
  const cache = getModuleSystem().c();
  if (cache[moduleId]) {
    delete cache[moduleId];
    return true;
  } else {
    return false;
  }
}

const moduleSystem = getModuleSystem();
moduleSystem.u = updateModule;
moduleSystem.e = deleteModule;
moduleSystem.a = clearCache;