<template>
  <div class="w-full h-full flex p-2 gap-3 box-border overflow-hidden">
    <template v-if="records.length > 0">
      <div class="min-w-fit flex flex-col border rounded-lg box-border">
        <div v-for="(item, index) in records"
          class="flex flex-col item-start justify-center cursor-pointer text-base font-semibold text-gray-900 px-3 py-4 border-b"
          :class="{ 'text-purple-500': current === index }" @click="current = index">
          <div class="flex">
            <div>{{ item.plugin }}</div>
            <div v-if="item.isHmr" class="ml-2 border text-purple-500 border-purple-500 px-2 rounded-md">
              hmr
            </div>
          </div>
          <div class="flex text-14px text-gray-400">
            <span>deps: {{ item.deps?.length || 0 }}</span>
          </div>
        </div>
      </div>
      <template v-if="deps.length > 0">
        <div class="w-full flex flex-col border rounded-lg box-border">
          <div v-for="item in deps"
            class="flex flex-col item-start justify-center cursor-pointer text-base font-semibold text-gray-900 px-3 py-4 border-b">
            <div>{{ item.source }}</div>
            <div class="text-sm text-gray-500">{{ item.kind }}</div>
          </div>
        </div>
      </template>
      <template v-else>
        <div class="w-full h-full flex items-center justify-center border rounded-lg box-border">
          no deps
        </div>
      </template>
    </template>
    <template v-else>
      <div class="w-full h-full flex items-center justify-center">no records</div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { AnalyzeDepsRecord } from 'farm/binding';
import { getAnalyzeDepsRecordsById } from '../api';
import { ref, watch, computed } from 'vue';
const props = defineProps({
  moduleId: String
});

const records = ref<AnalyzeDepsRecord[]>([]);
const current = ref(0);
const deps = computed(() => {
  if (records.value.length) {
    return records.value[current.value].deps;
  } else {
    return [];
  }
});
watch(
  () => props.moduleId,
  () => {
    current.value = 0;
    getAnalyzeDepsRecordsById(props.moduleId).then((res) => {
      records.value = res;
    });
  }
);
</script>
