<template>
  <article class="route-page">
    <p data-route-key="about" class="title">route:about</p>
    <p :data-message="content">{{ content }}</p>
    <p data-url="/about">{{ route.path }}</p>
  </article>
</template>

<script setup lang="ts">
import { useRoute } from "vue-router";
import { getRouteByKey } from "../../smoke/routes.mjs";

const route = useRoute();
const content = getRouteByKey("about")?.content ?? "about-route-missing";
</script>

<style scoped lang="less">
.title {
  color: red;
}
</style>
