<script setup lang="ts">
import HelloWorld from '../components/HelloWorld.vue'
import Formatter from '../components/Formatter.vue';
import IconComponent from '../assets/vue.svg?component'
</script>

<template>
  <el-button type="primary" @click="$router.push('/about')">to about page</el-button>

  <div class="container" flex item-center>
    <a href="https://farmfe.org/" target="_blank">
      <div class="logo1" />
      <div class="logo2" />
    </a>
    <a href="https://farmfe.org/" target="_blank">
      <img src="/logo.png" class="logo" alt="Farm logo" />
    </a>
    <a href="https://vuejs.org/" target="_blank">
      <my-svg-icon name="icon-vue" class="logo" style="height: 6.25rem; width: 6.25rem;" />
    </a>

    <a href="https://vuejs.org/" target="_blank">
      vite-svg-loader component
      <IconComponent class="logo" h-100px w-100px />
    </a>
  </div>

  <el-config-provider :size="'large'" :z-index="3000">
    <HelloWorld msg="Farm + Vue" />
    <Formatter />
  </el-config-provider>
</template>

<style scoped src="./index.scss"></style>

<style scoped lang="scss">
@import "bootstrap/scss/bootstrap";

aside {
  width: 100%;
  @include media-breakpoint-up(md) {
    width: 18rem;
  }
}

.h-full {
  height: calc(100vh - 4rem);
  /* max-width: calc(-webkit-fill-available - 4rem); */
}
</style>

<style scoped lang="less">
.container {
  .logo {
    height: 100px;
    padding: 18px;
    will-change: filter;
    transition: filter 300ms;
  }

  .logo1 {
    background-image: url('../assets/sub/logo.png');
    width: 100px;
    height: 100px;
    background-size: contain;
    display: inline-block;
  }

  .logo2 {
    background-image: url('../assets/sub/logo.png');
    width: 100px;
    height: 100px;
    background-size: contain;
    display: inline-block;
  }
}


.logo:hover {
  filter: drop-shadow(0 0 2em #9F1A8Faa);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>
