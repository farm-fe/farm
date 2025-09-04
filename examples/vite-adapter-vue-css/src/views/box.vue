<template>
  <div class="box"></div>

  <TailwindComponent />
</template>
<script setup lang="ts">
import TailwindComponent from './tailwind';

import "./box.css";
</script>
<style>
.box {
  width: 100px;
  height: 100px;
  background-color: red;
}
</style>
