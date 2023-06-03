<template>
  <Welcome />
</template>

<script lang="ts" setup>
import Welcome from './components/Welcome.vue';
const params = new URLSearchParams({
  date: '10/1',
  key: '8298b391cd106a1617d786fa2589af8f',
});

fetch(`/api?${params}`, {
  method: 'GET',
  headers: {
    'Content-Type': 'application/x-www-form-urlencoded',
  },
})
  .then((response) => response.json())
  .then((json) => console.log(json))
  .catch((error) => console.error(error));
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
