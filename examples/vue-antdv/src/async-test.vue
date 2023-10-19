<template>
  <Suspense>
    <Test />

    <template #fallback>
      Loading...
    </template>
  </Suspense>
</template>

<script lang="ts" setup>
import { Suspense } from 'vue';
import Test from './test.vue';
</script>