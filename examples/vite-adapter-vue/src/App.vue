
<template>
  <div>
    <test />
    <test1 />
  </div>
  <router-view />
</template>

<script lang="ts" setup>
import test1 from './components/test1.vue';
import test from './components/test.vue';
</script>

<style lang="scss" scoped>
body {
  // not work
  // background: red;
}
<style>
