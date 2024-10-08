<template>
  <div>
    <img src="/logo.png" alt="" />
    <test />
    <test1 />
    <aboute />
    <home />
  </div>
  <router-view />
</template>

<script lang="ts" setup>
import test1 from "./components/test1.vue";
import test from "./components/test.vue";
import home from "./pages/index.vue";
import aboute from "../deps/node_modules/my-ui/index.vue";

import "./test";
</script>

<style lang="scss" scoped>
body {
  // not work
  // background: red;
}
</style>
