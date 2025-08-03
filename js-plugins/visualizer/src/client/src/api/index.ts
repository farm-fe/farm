import { Module } from '@farmfe/core/binding/binding';
import { Resource } from 'farm';
import type { FarmEnvInfo } from '../../../node/utils/envinfo';
import { http } from '../http';

export function getModules(): Promise<Module[]> {
  return http.get<Module[]>('/__record/modules');
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

export function getPluginStats(): Promise<string> {
  return http.get('/__record/stats');
}
