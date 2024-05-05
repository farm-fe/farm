import { PluginStats, Resource } from '@farmfe/core';
import {
  AnalyzeDepsRecord,
  Module,
  ModuleRecord,
  ResolveRecord,
  ResourcePotRecord,
  TransformRecord
} from '@farmfe/core/binding';
import { FarmEnvInfo } from '../../../node/utils/envinfo';
import { http } from '../http';

export function getModules(): Promise<Module[]> {
  return http.get<Module[]>('/__record/modules');
}

export function getResolveRecordsById(id?: string): Promise<ResolveRecord[]> {
  return http.get<ResolveRecord[]>('/__record/resolve', {
    id
  });
}

export function getTransformRecordsById(
  id?: string
): Promise<TransformRecord[]> {
  return http.get<TransformRecord[]>('/__record/transform', {
    id
  });
}
export function getProcessRecordsById(id?: string): Promise<ModuleRecord[]> {
  return http.get<ModuleRecord[]>('/__record/process', {
    id
  });
}
export function getAnalyzeDepsRecordsById(
  id?: string
): Promise<AnalyzeDepsRecord[]> {
  return http.get<AnalyzeDepsRecord[]>('/__record/analyze_deps', {
    id
  });
}
export function getResourcePotRecordsById(
  id?: string
): Promise<ResourcePotRecord[]> {
  return http.get<ResourcePotRecord[]>('/__record/resource_pot', {
    id
  });
}

export function getFarmEnvInfo(): Promise<FarmEnvInfo> {
  return http.get('/__record/farm_env_info');
}

export function getResourcesMap(): Promise<Record<string, Resource>> {
  return http.get('/__record/resources_map');
}

export function getResource(id: string): Promise<string> {
  return http.get('/__record/resource', {
    id
  });
}

export function getPluginStats(): Promise<PluginStats> {
  return http.get('/__record/stats');
}
