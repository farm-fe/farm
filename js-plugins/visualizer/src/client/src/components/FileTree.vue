<template>
  <div class="flex w-full h-full overflow-scroll">
    <Tree :blockNode="true" :showLine="true" :tree-data="treeData">
      <template #title="{ key, title, isLeaf }">
        <div class="flex items-center justify-between w-full pt-[4px]">
          <div>{{ title }}</div>
          <div class="flex" v-if="isLeaf && getModuleInfo(key)">
            <Button
              size="small"
              class="flex justify-center items-center mr-2"
              @click="viewSourceCode(title, getModuleInfo(key)?.renderedContent)"
            >
              <CodepenCircleFilled />
            </Button>
            <Tag color="green"
              >Origin:{{ formatSize(getModuleInfo(key)?.originalLength) }}</Tag
            >
            <Tag color="blue"
              >Render:{{ formatSize(getModuleInfo(key)?.renderedLength) }}</Tag
            >
          </div>
        </div>
      </template>
    </Tree>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent } from 'vue';
import { Tree, Tag, Button } from 'ant-design-vue';
import { CodepenCircleFilled } from '@ant-design/icons-vue';
import { useResourcePotStore } from '../stores/resourcePot';
import { genFileTree } from '../utils/file';
import { formatSize } from '../utils/size';

export default defineComponent({
  name: 'FileTree',
  components: { Tree, Tag, CodepenCircleFilled, Button },
  setup(_, { emit }) {
    const resourcePotStore = useResourcePotStore();
    const treeData = computed(() => {
      return genFileTree(resourcePotStore.moduleIds);
    });

    function getModuleInfo(key: string) {
      if (resourcePotStore.modules) {
        return resourcePotStore.modules[key];
      } else {
        return null;
      }
    }
    function viewSourceCode(name: string, code?: string) {
      emit('view', { name, code });
    }
    return {
      treeData,
      getModuleInfo,
      resourcePotStore,
      formatSize,
      viewSourceCode
    };
  }
});
</script>
