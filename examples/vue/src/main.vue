<template>
  <div className='button-wrapper'>
    <CounterButton />
  </div>
  <div>
    <Description />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import CounterButton from './comps/counter-button/index.vue';
import Description from './comps/description/index.vue';
import './main.css';

export default defineComponent({
  components: { CounterButton, Description }
})
</script>

<style></style>