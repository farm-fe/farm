<template>
  <button @click="changeColor">change color</button>

  <div class="comp">
    <component :is="comp" />
  </div>
</template>

<script lang="tsx" setup>
import { ref, computed } from 'vue';
// button 0
const currentColor = ref('red');

const comp = computed(() => (<div style={{ width: '100%', height: '100%', backgroundColor: currentColor.value }}></div>));

const changeColor = () => {
  if (currentColor.value === 'red') {
    currentColor.value = 'blue';
  } else {
    currentColor.value = 'red';
  }
}
</script>

<style scoped>
.comp {
  width: 100px;
  color: #fff;
  width: 100px;
  height: 30px;
}
</style>