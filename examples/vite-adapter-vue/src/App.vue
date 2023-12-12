<script setup lang="ts">
import HelloWorld from './components/HelloWorld.vue';
import Formatter from './components/Formatter.vue';
console.log('Hello from Vite + Vue + TypeScript + ESLint + Prettier');
</script>

<template>
  <div>
    <a href="https://farm-fe.github.io/" target="_blank">
      <img src="./assets/logo.png" class="logo" alt="Farm logo" />
    </a>
    <a href="https://vuejs.org/" target="_blank">
      <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
    </a>
  </div>
  <el-config-provider :size="'large'" :z-index="3000">
    <HelloWorld msg="Farm + Vue" />
    <Formatter />
  </el-config-provider>
</template>

<style scoped>
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: filter 300ms;
}

.logo:hover {
  filter: drop-shadow(0 0 2em #9f1a8faa);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>
