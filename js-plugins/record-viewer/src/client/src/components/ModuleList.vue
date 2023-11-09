<template>
  <div class="border shadow-sm rounded-lg p-3 box-border overflow-y-scroll">
    <div
      v-for="item in filterList"
      class="text-base font-semibold px-2 py-3 border-b color-gray-900 hover:text-purple-600"
      :class="{ 'text-purple-600': props.moduleId === item.id }"
      @click="handleClick(item)"
    >
      <div class="text-lg">{{ item.id }}</div>
      <!-- <div class="flex mt-2 text-gray-600">
        resource pot:
        <span class="w-fit-content ml-2 border text-purple-500 border-purple-500 px-2 rounded-md">
          {{ item.resourcePot || 'None' }}
        </span>
      </div> -->
      <div class="text-sm font-normal text-gray-400">{{ item.moduleType }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { getModules } from "../api";
import { Module } from "@farmfe/core/binding";

const props = defineProps({
  moduleId: String,
});

const emit = defineEmits(["click"]);
const moduleList = ref<Module[]>([]);
const filterList = computed(() => {
  return moduleList.value.filter((item) => !item.immutable);
});

function handleClick(item: Module) {
  emit("click", item);
}

getModules().then((res) => {
  console.log("modules:", res);

  moduleList.value = res;
});
</script>
