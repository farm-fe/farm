<template>
  <div>
    <DatePicker v-model:value="selectedDate" valueFormat="YYYY-MM-DD" />
    <span style="margin-left: 10px;">current selected date: {{ selectedDate ?? '-' }}</span>

    <div>
      <p>dynamic import:</p>
      <p>default: {{ d() }}</p>
      <p>named: {{ n() }}</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { DatePicker } from 'ant-design-vue';

const selectedDate = ref<string>();

const { default: d, named: n } = await import('./dynamic-data');

</script>