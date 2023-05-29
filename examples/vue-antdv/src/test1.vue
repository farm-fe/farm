<template>
  <div>
    test1
    <RouterView />
  </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { DatePicker } from 'ant-design-vue';

const selectedDate = ref<string>();
</script>