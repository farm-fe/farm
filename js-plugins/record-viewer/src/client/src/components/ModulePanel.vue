<template>
  <div class="flex flex-col border rounded-lg box-border overflow-hidden">
    <div class="text-lg font-semibold text-gray-900 border-b py-4 px-3">
      <span>{{ moduleId }}</span>
    </div>
    <div class="flex flex-col w-full h-full p-2 box-border overflow-hidden">
      <div class="flex items-center border-b-3">
        <div class="cursor-pointer px-5 py-1 hover:text-purple-600" :class="[currentTab === RecordTypes.Resolve ? 'text-purple-600': '']" @click="currentTab = RecordTypes.Resolve">{{
          RecordTypes.Resolve }}</div>
        <div class="cursor-pointer px-5 py-1 hover:text-purple-600" :class="[currentTab === RecordTypes.Transform ? 'text-purple-600': '']" @click="currentTab = RecordTypes.Transform">{{ RecordTypes.Transform }}</div>
        <div class="cursor-pointer px-5 py-1 hover:text-purple-600" :class="[currentTab === RecordTypes.Process ? 'text-purple-600': '']" @click="currentTab = RecordTypes.Process">{{ RecordTypes.Process }}</div>
        <div class="cursor-pointer px-5 py-1 hover:text-purple-600" :class="[currentTab === RecordTypes.AnalyzeDeps ? 'text-purple-600': '']" @click="currentTab = RecordTypes.AnalyzeDeps">{{ RecordTypes.AnalyzeDeps }}</div>
      </div>
      <ResolveRecord v-show="currentTab === RecordTypes.Resolve" :module-id="moduleId"></ResolveRecord>
      <TransformRecord  v-show="currentTab === RecordTypes.Transform" :module-id="moduleId"></TransformRecord>
      <ProcessRecord  v-show="currentTab === RecordTypes.Process" :module-id="moduleId"></ProcessRecord>
      <AnalyzeDepsRecord v-show="currentTab === RecordTypes.AnalyzeDeps" :module-id="moduleId"></AnalyzeDepsRecord>
    </div>
  </div>
</template>
<script lang="ts" setup>
import { watch, ref } from "vue";
import TransformRecord from "./TransformRecord.vue";
import ResolveRecord from "./ResolveRecord.vue";
import ProcessRecord from "./ProcessRecord.vue";
import AnalyzeDepsRecord from "./AnalyzeDepsRecord.vue";

const props = defineProps({
  moduleId: String,
});

enum RecordTypes {
  Resolve = "Resolve Records",
  Transform = "Transform Records",
  Process = "Process Records",
  AnalyzeDeps = "Analyze Deps Records",
  ResourcePot = "ResourcePot Records",
}

const currentTab = ref(RecordTypes.Resolve);
// const recordTabs = ref([
//   'Resolve Records',
//   'Transform Records',
//   'Process Records',
//   'Analyze Deps Records',
//   'ResourcePot Records'
// ])

watch(
  () => props.moduleId,
  () => {
    currentTab.value = RecordTypes.Resolve;
  }
);


</script>

