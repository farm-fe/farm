<script setup lang="ts">
import HelloWorld from './components/HelloWorld.vue'
import IconAccessibility from '~icons/fontisto/ambulance'
import LocalIcon from '~icons/local/vue'
import RemoteIcon from "~icons/remote/simpleicons?width=6em&height=6em&fill=#42b883"
</script>

<template>
  <div>
    <a href="https://farmfe.org/" target="_blank">
      <img src="./assets/logo.png" class="logo" alt="Farm logo" />
    </a>
    <a href="https://vuejs.org/" target="_blank">
      <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
    </a>
  <IconAccessibility color="red" />
  <LocalIcon style="height: 8em; width: 8em;" />
  <RemoteIcon />
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
  filter: drop-shadow(0 0 2em #9F1A8Faa);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>
