import type { ModuleInitialization, ModuleSystem } from "../module-system.js";

let moduleSystem: ModuleSystem;

export function initModuleSystem(ms: ModuleSystem) {
  moduleSystem = ms;
  moduleSystem.u = updateModule;
  moduleSystem.e = deleteModule;
  moduleSystem.a = clearCache;
}

function updateModule(moduleId: string, init: ModuleInitialization): void {
  const modules = moduleSystem.m();
  modules[moduleId] = init;
  clearCache(moduleId);
}

function deleteModule(moduleId: string): boolean {
  const modules = moduleSystem.m();
  if (modules[moduleId]) {
    clearCache(moduleId);
    delete modules[moduleId];
    return true;
  } else {
    return false;
  }
}

function clearCache(moduleId: string): boolean {
  const cache = moduleSystem.c();
  if (cache[moduleId]) {
    delete cache[moduleId];
    return true;
  } else {
    return false;
  }
}