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
        <div></div>
      </div>
    </Card>
  </div>
</template>

<script lang="ts" setup>
import { Card } from "ant-design-vue";
import { ref } from "vue";
import ModuleList from "../../components/ModuleList.vue";
import type { Module } from "@farmfe/core";

const currentModule = ref<Module>();
function handleSelect(module: Module) {
  currentModule.value = module;
}

</script>
