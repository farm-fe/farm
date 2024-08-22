<script setup lang="ts">
import { NButton, NFlex } from "naive-ui";

const open = () => {
  alert("Clicked");
};
</script>

<template>
  <n-flex class="fn-buttons">
    <n-button type="primary" size="small" @click="open()">Click Me</n-button>
  </n-flex>
</template>

<style scoped>
.fn-buttons {
  margin: 5px;
}
</style>
