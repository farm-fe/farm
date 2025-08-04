<template>
  <div class="p-3 border box-border rounded-lg w-full h-full overflow-y-scroll">
    <template v-if="records.length > 0">
      <div v-for="item in records" class="text-gray-800 px-3 py-4 border-b text-base font-semibold">
        <div class="flex">
          <div>{{ item.plugin }}</div>
          <div v-if="item.hook === 'parse'" class="ml-2 border text-purple-500 border-purple-500 px-2 rounded-md">
            parse
          </div>
          <div v-if="item.isHmr" class="ml-2 border text-purple-500 border-purple-500 px-2 rounded-md">
            hmr
          </div>
        </div>
        <div class="text-sm text-gray-500">{{ item.moduleType }}</div>
      </div>
    </template>
    <template v-else>
      <div class="w-full h-full flex items-center justify-center">no records</div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { ModuleRecord } from 'farm/binding';
import { getProcessRecordsById } from '../api';
import { ref, watch } from 'vue';
const props = defineProps({
  moduleId: String
});

const records = ref<ModuleRecord[]>([]);
watch(
  () => props.moduleId,
  () => {
    getProcessRecordsById(props.moduleId).then((res) => {
      records.value = res;
    });
  }
);
</script>
