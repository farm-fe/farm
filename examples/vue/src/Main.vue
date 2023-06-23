<template>
  <Welcome />
</template>

<script lang="ts" setup>
import Welcome from './components/Welcome.vue';
import axios from "axios"
console.log(axios.create);
fetch('/api')
  .then((response) => response.json())
  .then((json) => console.log(json));
</script>

<style>
#app {
  font-size: 18px;
  display: flex;
  justify-content: center;
  max-width: 1280px;
  margin: 0 auto;
  font-weight: normal;
}
</style>
