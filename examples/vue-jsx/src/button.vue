<template>
  <button @click="changeColor">change color</button>

  <component :is="comp" />
</template>

<script lang="tsx" setup>
import { ref } from 'vue';

const comp = ref(<div style={{ color: '#fff', width: '100px', height: '30px', backgroundColor: 'red' }}></div>);

const changeColor = () => {
  comp.value = <div style={{ color: '#fff', width: '100px', height: '30px', backgroundColor: 'blue' }}></div>;
}
</script>