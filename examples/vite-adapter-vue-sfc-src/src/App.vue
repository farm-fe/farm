<script setup lang="ts">
import HelloWorld from './components/HelloWorld.vue';
import DeferImport from './components/DeferImport.vue';
</script>

<template>
  <div>
    <a href="https://farmfe.org/" target="_blank">
      <img src="./assets/logo.png" class="logo" alt="Farm logo" />
    </a>
    <a href="https://vuejs.org/" target="_blank">
      <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
    </a>
  </div>
  <HelloWorld msg="Farm + Vue" />
  <DeferImport />
</template>

<!-- <style scoped src="./app.css"></style> -->
<style scoped src="./app.css"></style>
<style scoped>
/* 213 */
</style>
