<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useCounterStore } from '../stores/counter';

const counter = useCounterStore();
const { count } = storeToRefs(counter);
</script>

<template>
  <article class="card">
    <strong>Pinia count: {{ count }}</strong>
    <button type="button" @click="counter.increment()">Increment</button>
  </article>
</template>

<style lang="scss" scoped>
$accent: #40916c;

.card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem;
  border-radius: 12px;
  background: rgba($accent, 0.12);

  button {
    border: 0;
    border-radius: 999px;
    padding: 0.5rem 0.85rem;
    color: white;
    background: $accent;
  }
}
</style>
