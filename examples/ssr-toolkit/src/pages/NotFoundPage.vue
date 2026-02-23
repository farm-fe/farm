<template>
  <article class="route-page route-not-found">
    <p data-route-key="not-found">route:not-found</p>
    <p :data-message="missingMessage">{{ missingMessage }}</p>
    <p :data-url="missingPath">{{ missingPath }}</p>
    <p id="route-not-found" :data-missing-path="missingPath">
      Route Not Found: {{ missingPath }}
    </p>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();
const missingPath = computed(() => route.path || '/');
const missingMessage = computed(() => `route-not-found:${missingPath.value}`);
</script>
