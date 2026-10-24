<script setup lang="ts">
defineProps<{ stock: number }>();
</script>
<template>
  <span class="stock-badge" :class="{ low: stock < 25 }">{{ stock < 25 ? 'Low stock' : 'Healthy' }}</span>
</template>
<style scoped>
.stock-badge { padding: 4px 8px; border-radius: 12px; color: #194e3e; background: #dcecdf; }
.stock-badge.low { color: #704c11; background: #f4e3b9; }
</style>
