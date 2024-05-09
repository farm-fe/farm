<template>
  <div class="flex flex-col w-screen h-screen rounded-4px">
    <NavBar class="w-full fixed top-0 z-10"></NavBar>
    <main class="mt-16 w-full h-full p-6 bg-gray-200 overflow-scroll">
      <router-view></router-view>
    </main>
  </div>
</template>

<script setup lang="ts">
import NavBar from './components/NavBar.vue';

// import Dashboard from './components/Dashboard.vue';
</script>
