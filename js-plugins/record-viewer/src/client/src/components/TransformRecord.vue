<template>
  <div class="w-full h-full flex p-2 gap-3 box-border overflow-hidden">
    <div class="w-fit flex flex-col border rounded-lg box-border">
      <div v-for="(item, index) in records" class="flex flex-col item-start justify-center cursor-pointer text-base font-semibold text-gray-900 px-3 py-4 border-b" :class="{'text-purple-500': current === index}" @click="current = index">
        <div class="flex">
        <div>{{ item.plugin }}</div>
        <div v-if="item.hook === 'load'" class="ml-2 border text-purple-500 border-purple-500 px-2 rounded-md">load</div>
        <div v-if="item.isHmr" class="ml-2 border text-purple-500 border-purple-500 px-2 rounded-md">hmr</div>
        </div>
        <div class="text-14px text-gray-400">{{item.moduleType}}</div>
      </div>
    </div>
    <code-diff style="margin-top: 0px; margin-bottom:0px;" :old-string="compareRecord.old?.content" :new-string="compareRecord.new?.content" output-format="side-by-side" />
  </div>
</template>

<script lang="ts" setup>
import { TransformRecord } from "@farmfe/core/binding";
import { CodeDiff } from 'v-code-diff'
import { getTransformRecordsById } from "../api";
import { ref, watch, computed } from "vue";
const props = defineProps({
  moduleId: String,
});

const records = ref<TransformRecord[]>([]);
const current = ref(0)

const compareRecord = computed(() => {
  return {
    old: records.value[current.value - 1],
    new: records.value[current.value]
  }
})
watch(
  () => props.moduleId,
  () => {
    current.value = 0
    getTransformRecordsById(props.moduleId).then((res) => {
      console.log("res", res);
      records.value = res;
    });
  }
);
</script>
