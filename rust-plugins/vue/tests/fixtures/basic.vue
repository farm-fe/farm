<template>
  <div class="greeting">{{ msg }}</div>
</template>

<script setup lang="ts">
const msg: string = 'hello from fervid'
</script>

<style scoped>
.greeting {
  color: red;
}
</style>
