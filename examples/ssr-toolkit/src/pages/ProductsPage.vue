<template>
  <article class="route-page">
    <p data-route-key="products">route:products</p>
    <p :data-message="content">{{ content }}</p>
    <p data-url="/products">{{ route.path }}</p>
  </article>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import { getRouteByKey } from '../../smoke/routes.mjs';

const route = useRoute();
const content = getRouteByKey('products')?.content ?? 'products-route-missing';
</script>
