<template>
  <div class="flex">
    <Card
      class="w-full overflow-hidden"
      title="Module Analysis"
      :body-style="{ flex: '1' }"
    >
      <div class="flex gap-x-5 w-full max-h-[70vh]">
        <!-- Module List -->
        <ModuleList class="w-1/3" @select="handleSelect"></ModuleList>
        <ModuleRecords
          class="w-2/3 flex flex-col"
          :module-id="currentModule?.id"
        ></ModuleRecords>
      </div>
    </Card>
  </div>
</template>

<script lang="ts">
import { Card, Tree, Drawer } from "ant-design-vue";
import { defineComponent, ref } from "vue";
import FileTree from "../../components/FileTree.vue";
import ResourcePots from "../../components/ResourcePots.vue";
import CodeViewer from "../../components/CodeViewer.vue";
import ModuleList from "../../components/ModuleList.vue";
import ModuleRecords from "../../components/ModuleRecords.vue";
import type { Module } from "@farmfe/core";

export default defineComponent({
  name: "ModuleAnalyze",
  components: {
    Card,
    Tree,
    ResourcePots,
    FileTree,
    CodeViewer,
    Drawer,
    ModuleList,
    ModuleRecords,
  },
  setup() {
    const currentModule = ref<Module>();
    function handleSelect(module: Module) {
      currentModule.value = module;
    }

    return {
      handleSelect,
      currentModule,
    };
  },
});
</script>
