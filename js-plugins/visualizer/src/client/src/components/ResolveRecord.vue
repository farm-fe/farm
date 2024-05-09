<template>
  <Card class="w-full h-full" :body-style="{ height: '100%', overflow: 'scroll' }">
    <div
      v-for="item in records"
      class="text-gray-800 px-3 py-4 border-b text-base font-semibold"
    >
      <div>{{ item.importer || "None" }}</div>
      <div class="text-sm text-gray-500">{{ item.plugin }} | {{ item.kind }}</div>
    </div>
  </Card>
</template>

<script lang="ts" setup>
import { ResolveRecord } from '@farmfe/core/binding';
import { getResolveRecordsById } from '../api';
import { ref, watch } from 'vue';
import { Card } from 'ant-design-vue';
const props = defineProps({
  moduleId: String
});

const records = ref<ResolveRecord[]>([]);
watch(
  () => props.moduleId,
  () => {
    getResolveRecordsById(props.moduleId).then((res) => {
      records.value = res;
    });
  }
);
</script>
