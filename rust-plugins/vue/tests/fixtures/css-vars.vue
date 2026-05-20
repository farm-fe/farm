<template>
  <div class="box" />
</template>

<script setup>
const color = 'red'
</script>

<style scoped>
.box {
  color: v-bind(color);
}
</style>
