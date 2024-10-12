<template>
  <div>
    <img src="/logo.png" alt="">
    <test />

    <TinyButton type="primary" @click="btnClick">
      Tiny Vue Modal 最大化显示
    </TinyButton>
    
    <test1 />
    <aboute />
    <home/>
  </div>
  <router-view />
</template>

<script lang="ts" setup>
import { Button as TinyButton, Modal } from '@opentiny/vue'

function btnClick() {
  Modal.alert({ message: '最大化显示', fullscreen: true })
}

import test1 from './components/test1.vue';
import test from './components/test.vue';
import home from './pages/index.vue';
import aboute from '../deps/node_modules/my-ui/index.vue'

import './test';
</script>

<style lang="scss" scoped>
body {
  // not work
  // background: red;
}
</style>
