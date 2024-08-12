<script setup lang="ts">
import { ref } from "vue";
import HelloWorld from "./components/HelloWorld.vue";
const a = ref(666);

const incrementA = () => {
  a.value++;
};

const getMessage = () => {
  return a.value % 2 === 0 ? "a 是偶数" : "a 是奇数";
};

fetch(
  "https://apis.juhe.cn/environment/river?key=caab07f6c5df752d2e28edfe447ae6d0",
)
  .then((res) => res.json())
  .then((data) => console.log(data));
</script>

<template>
  <div>
    <a href="https://farmfe.org/" target="_blank">
      <img src="./assets/logo.png" class="logo" alt="Farm logo" />
    </a>
    {{ a }}
    <button @click="incrementA">增加 a</button>
    <p>{{ getMessage() }}</p>
    <a href="https://vuejs.org/" target="_blank">
      <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
    </a>
  </div>
  <HelloWorld msg="Farm + Vue" />
</template>

<style scoped>
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: filter 300ms;
}

.logo:hover {
  filter: drop-shadow(0 0 2em #9f1a8faa);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>
