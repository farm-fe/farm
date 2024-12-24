<script setup lang="ts">
import HelloWorld from "@/components/HelloWorld.vue";
// console.log(import.meta.env);
import { bbb } from "./indexa.ts";

bbb();
fetch("https://wallhaven.fun/api/wallhaven/w/yx6e9l").then((res) => res.json());
// .then((data) => console.log(data))
// .catch((err) => console.error(err));+

// window.onbeforeunload = function () {
//   fetch("http://localhost:3000", { method: "post" });
//   navigator.sendBeacon(
//     "http://localhost:3000",
//     JSON.stringify({ key: "value" })
//   );
// };

// const aa = ref(92922299)
// console.log(aa.value);

</script>

<template>
  <div>
    <a href="https://farmfe.org/" target="_blank">
      <img src="./assets/logo.png" class="logo" alt="Farm logo" /> </a>
    <a href="https://vuejs.org/" target="_blank">
      <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
    </a>
    <div>
    <img width="500" src="/aaa.jpg" alt="">
    </div>
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
