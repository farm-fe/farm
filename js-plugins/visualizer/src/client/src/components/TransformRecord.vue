<template>
  <div class="w-full h-full flex p-2 gap-3 box-border overflow-hidden">
    <template v-if="records.length > 0">
      <Card class="w-full h-full" :body-style="{ height: '100%', overflow: 'scroll' }">
        <div
          v-for="(item, index) in records"
          class="flex flex-col item-start justify-center cursor-pointer text-base font-semibold text-gray-900 px-3 py-4 border-b"
          :class="{ 'text-purple-500': current === index }"
          @click="selectRecord(index)"
        >
          <div class="flex">
            <div>{{ item.plugin }}</div>
            <div
              v-if="item.hook === 'load'"
              class="ml-2 border text-purple-500 border-purple-500 px-2 rounded-md"
            >
              load
            </div>
            <div
              v-if="item.isHmr"
              class="ml-2 border text-purple-500 border-purple-500 px-2 rounded-md"
            >
              hmr
            </div>
          </div>
          <div class="text-14px text-gray-400">{{ item.moduleType }}</div>
        </div>
      </Card>
    </template>
    <template v-else>
      <div class="w-full h-full flex items-center justify-center">no records</div>
    </template>
    <Drawer v-model:open="isOpened" :title="moduleId" placement="right" width="80vw">
      <CodeDiff :original="compareRecord.old?.content" :modified="compareRecord.new.content" :language="'javascript'"></CodeDiff>
    </Drawer>
  </div>
</template>

<script lang="ts" setup>
import { TransformRecord } from '@farmfe/core/binding';
import { Card, Drawer } from 'ant-design-vue';
import { getTransformRecordsById } from '../api';
import { ref, watch, computed } from 'vue';
import CodeDiff from './CodeDiff.vue';
const props = defineProps({
  moduleId: String
});

const records = ref<TransformRecord[]>([]);
const current = ref(0);
const isOpened = ref(false);

const compareRecord = computed(() => {
  return {
    old: records.value[current.value - 1] || null,
    new: records.value[current.value]
  };
});

function selectRecord(index: number) {
  current.value = index;
  isOpened.value = true;
}
watch(
  () => props.moduleId,
  () => {
    current.value = 0;
    getTransformRecordsById(props.moduleId).then((res) => {
      records.value = res;
    });
  }
);
</script>
