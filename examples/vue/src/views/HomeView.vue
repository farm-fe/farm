<script setup lang="ts">
import CounterCard from '../components/CounterCard.vue';
import Welcome from '../components/Welcome';
</script>

<template>
  <section>
    <p class="intro">This example uses the native Rust <code>@farmfe/plugin-vue</code>.</p>
    <Welcome />
    <CounterCard />
  </section>
</template>

<style scoped>
.intro {
  color: #2d6a4f;
}
</style>

<style>
.welcome-jsx {
  margin: 1rem 0;
}

.jsx-badge {
  color: #1b4332;
  font-size: 0.9rem;
  margin-bottom: 0.5rem;
}

.jsx-card {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
  padding: 1rem;
  border: 1px solid #95d5b2;
  border-radius: 8px;
  background: rgba(45, 106, 79, 0.08);
  margin-bottom: 1rem;
}

.jsx-card button {
  padding: 0.25rem 0.75rem;
  border: 1px solid #40916c;
  border-radius: 4px;
  background: #40916c;
  color: #fff;
  font-size: 0.85rem;
  cursor: pointer;
}

.jsx-reveal {
  width: 100%;
  color: #2d6a4f;
  font-weight: 500;
}
</style>
