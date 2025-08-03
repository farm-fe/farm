<template>
  <div class="flex">
    <Card class="w-full overflow-hidden" title="Bundle Analysis">
      <div class="flex gap-x-5 w-full max-h-[70vh]">
        <!-- Resource Pots List -->
        <ResourcePots class="w-1/3" :pots="resourcePots" @view="handleViewCode"></ResourcePots>
        <Card :title="resourcePotStore.resource?.name
            ? `Modules of ${resourcePotStore.resource?.name}`
            : 'Modules'
          " class="w-2/3 flex flex-col" :body-style="{ overflow: 'hidden' }">
          <div class="text-sm text-gray-500 mb-1">
            Modules: {{ resourcePotStore.moduleIds?.length || 0 }}
          </div>
          <FileTree @view="handleViewCode"></FileTree>
        </Card>
      </div>
    </Card>
    <Drawer v-model:open="isOpened" :title="sourceFile?.name" placement="right" width="80vw">
      <CodeViewer :code="sourceFile.code" :language="'javascript'"></CodeViewer>
    </Drawer>
  </div>
</template>

<script lang="ts">
import { Card, Tree, Drawer } from 'ant-design-vue';
import { computed, defineComponent, ref, reactive } from 'vue';
import { getResourcesMap } from '../../api';
import type { Resource } from 'farm';
import ResourcePots from '../../components/ResourcePots.vue';
import { useResourcePotStore } from '../../stores/resourcePot';
import { genFileTree } from '../../utils/file';
import FileTree from '../../components/FileTree.vue';
import CodeViewer from '../../components/CodeViewer.vue';

export default defineComponent({
  name: 'BundleAnalyze',
  components: { Card, Tree, ResourcePots, FileTree, CodeViewer, Drawer },
  setup() {
    const resourcePotStore = useResourcePotStore();
    const resourcePots = ref<Resource[]>([]);
    const isOpened = ref(false);
    const sourceFile = reactive({
      name: '',
      code: '',
      language: 'javascript'
    });
    getResourcesMap().then((rawData) => {
      resourcePots.value = Object.values(rawData);
    });
    const treeData = computed(() => {
      const moduleIds = Object.keys(resourcePotStore.modules || {});
      return genFileTree(moduleIds);
    });

    function handleViewCode(data: {
      name: string;
      code: string;
      language?: string;
    }) {
      sourceFile.name = data.name;
      sourceFile.code = data.code;
      sourceFile.language = data.language || 'javascript';
      isOpened.value = true;
    }

    return {
      resourcePots,
      resourcePotStore,
      treeData,
      isOpened,
      sourceFile,
      handleViewCode
    };
  }
});
</script>
