<template>
  <main
    id="ssr-app"
    :data-url="route.fullPath"
    :data-route-path="routePath"
    :data-route-status="routeStatus"
    data-style-engine="less"
  >
    <header class="app-header">
      <h1>{{ pageHeading }}</h1>
      <nav class="app-nav">
        <a href="/">Home</a>
        <a href="/about">About</a>
        <a href="/products">Products</a>
      </nav>
    </header>
    <section class="route-shell">
      <router-view />
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, inject } from 'vue';
import { useRoute } from 'vue-router';

const fallbackBanner = 'Farm SSR Toolkit';
const homeBanner = inject('homeBanner', fallbackBanner);
const route = useRoute();

const routePath = computed(() => route.path || '/');
const routeStatus = computed(() =>
  route.meta?.status === 'not-found' ? 'not-found' : 'ok'
);
const pageHeading = computed(() => {
  if (route.meta?.key === 'home') {
    return homeBanner;
  }

  if (typeof route.meta?.heading === 'string') {
    return route.meta.heading;
  }

  return homeBanner;
});
</script>
