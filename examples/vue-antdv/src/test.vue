<template>
  <div>
    <DatePicker v-model:value="selectedDate" valueFormat="YYYY-MM-DD" />
    <span style="margin-left: 10px;">current selected date: {{ selectedDate ?? '-' }}</span>
  </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { DatePicker } from 'ant-design-vue';

const selectedDate = ref<string>();
</script>