<template>
  <a class="farm-button" :href="props.to" target="_blank">
    <slot></slot>
  </a>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  props: {
    to: String,
  },
  setup(props) {
    return {
      props,
    };
  },
});
</script>

<style>
.farm-button {
  background-color: #9f1a8f;
  color: #fff;
  padding: 10px 25px;
  border-radius: 10px;
  text-decoration: none;
}

.farm-button:hover {
  background-color: #bf1aaf;
  box-shadow: 0 4px 6px -1px rgb(0 0 0 / 10%), 0 2px 4px -2px rgb(0 0 0 / 10%);
}
</style>
