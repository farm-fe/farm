<template>
  <div class="flex p-4 box-border w-full h-full gap-3 overflow-hidden bg-#EEEDEB">
    <ModuleList class="min-w-10vw grow transition-all duration-300" @click="moduleClick" :module-id="current?.id"></ModuleList>
    <ModulePanel class="grow-2 shrink-0 overflow-hidden min-w-lg transition-all duration-300" v-show="current" :module-id="current?.id" style="max-width: 85vw;"></ModulePanel>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ModuleList from '../components/ModuleList.vue';
import ModulePanel from '../components/ModulePanel.vue';
import { Module } from '@farmfe/core/binding';

const current = ref<Module | undefined>(undefined)
function moduleClick(item: Module) {
  current.value = item
}
</script>

