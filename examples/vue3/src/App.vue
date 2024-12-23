<script setup lang="ts">
import { ref, watch } from 'vue'
const msg = ref('')
watch(msg, (newVal, oldVal) => {
  console.log('msg changed:', newVal, oldVal)
})
const count = ref(100)
</script>

<template>
  <div>
    <h2>{{ msg }}</h2>
    <input v-model="msg" type="text" />
    {{ count }}
    <button @click="count++">click</button>
  </div>

</template>

<style>
body {}
</style>
