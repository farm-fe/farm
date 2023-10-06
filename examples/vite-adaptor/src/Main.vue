<template>
  <Welcome />
</template>

<script lang="ts" setup>
import Welcome from './components/Welcome.vue';
</script>

<style lang="scss">
@import './styles/global.scss';

#app {
  font-size: 18px;
  display: flex;
  justify-content: center;
  max-width: $max-width;
  margin: 0 auto;
  font-weight: normal;
}
</style>
