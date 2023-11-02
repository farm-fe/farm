<script setup lang="ts">
import { ref } from 'vue';

defineProps<{ msg: string }>()
const count = ref(0)
console.log(count.value);
</script>

<template>
  <h1>{{ msg }}</h1>

  <div class="card">
    <el-button type="primary" @click="count++">count is {{ count }}</el-button>

    <p>
      Edit
      <code>components/HelloWorld.vue</code> to test HMR
    </p>
  </div>

  <p>
    Install
    <a href="https://github.com/vuejs/language-tools" target="_blank">Volar</a>
    in your IDE for a better DX
  </p>
  <p class="read-the-docs">Click on the Farm and Vue logos to learn more</p>
</template>

<style scoped lang="scss">
@forward 'element-plus/theme-chalk/src/mixins/config.scss';
@import 'element-plus/theme-chalk/src/upload.scss';

.read-the-docs {
  color: #888;
}
</style>
