import { RenderedModule, Resource } from 'farm';
import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { getResourcesMap } from '../api';

export const useResourcePotStore = defineStore('resourcePot', () => {
  const resourcePots = ref<Resource[]>([]);
  const resource = ref<Resource | undefined>(undefined);
  const moduleIds = computed(() => {
    return resource.value?.info?.moduleIds || [];
  });
  const modules = computed(() => {
    return resource.value?.info?.modules || null;
  });

  function getResourcePots() {
    return getResourcesMap().then((rawData) => {
      const res = Object.values(rawData);
      resourcePots.value = res;
      return res;
    });
  }

  function setResource(data: Resource) {
    resource.value = data;
  }

  return {
    getResourcePots,
    setResource,
    resourcePots,
    resource,
    moduleIds,
    modules
  };
});
