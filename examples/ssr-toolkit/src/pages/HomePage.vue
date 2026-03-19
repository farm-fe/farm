<template>
  <article class="route-page">
    <p data-route-key="home">route:home</p>
    <p :data-message="content">{{ content }}</p>
    <p data-url="/">{{ route.path }}</p>
    <p class="less-theme-active">less-theme-active</p>
  </article>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import { getRouteByKey } from '../routes.mjs';

const route = useRoute();
const content = getRouteByKey('home')?.content ?? 'home-route-missing';
</script>
