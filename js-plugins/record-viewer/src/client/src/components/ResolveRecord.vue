<template>
  <div class="p-3 border box-border rounded-lg w-full h-full overflow-y-scroll">
    <div v-for="item in records" class="text-gray-800 px-3 py-4 border-b text-base font-semibold ">
      <div>{{ item.importer  || "None"}}</div>
      <div class="text-sm text-gray-500">{{ item.plugin }} | {{item.kind}}</div>
    </div>
  </div>
</template>


<script lang="ts" setup>
import { ResolveRecord } from '@farmfe/core/binding';
import { getResolveRecordsById } from '../api';
import { ref, watch } from 'vue'
const props = defineProps({
  moduleId: String
})


const records = ref<ResolveRecord[]>([])
watch(() => props.moduleId, () => {
  getResolveRecordsById(props.moduleId).then(res => {
    records.value = res
  })
})




</script>
