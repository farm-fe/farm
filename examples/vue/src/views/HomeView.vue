<script setup lang="ts">
import CounterCard from '../components/CounterCard.vue';
</script>

<template>
  <section>
    <p class="intro">This example uses the native Rust <code>@farmfe/plugin-vue</code>.</p>
    <CounterCard />
  </section>
</template>

<style scoped>
.intro {
  color: #2d6a4f;
}
</style>
