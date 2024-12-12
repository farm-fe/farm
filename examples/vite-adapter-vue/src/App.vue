<template>
  <el-tabs v-model="activeName">
    <img src="/logo.png" alt="">
    <el-tab-pane label="test" name="test">
      <test />
    </el-tab-pane>
    <el-tab-pane label="test1" name="test1">
      <test1 />
    </el-tab-pane>
    <el-tab-pane label="home" name="home">
      <home />
    </el-tab-pane>
    <el-tab-pane label="aboute" name="aboute">
      <aboute />
    </el-tab-pane>

    <!-- <TinyButton type="primary" @click="btnClick">
      Tiny Vue Modal 最大化显示
    </TinyButton> -->

    <!-- <test1 />
    <aboute />
    <home/> -->
  </el-tabs>
  <router-view />
</template>

<script lang="ts" setup>
import { ElTabs, ElTabPane } from 'element-plus'
import 'element-plus/theme-chalk/src/tabs.scss'
import 'element-plus/theme-chalk/src/tab-pane.scss'
// import { Button as TinyButton, Modal } from '@opentiny/vue'

// function btnClick() {
//   Modal.alert({ message: '最大化显示', fullscreen: true })
// }

// import test1 from './components/test1.vue';
// import test from './components/test.vue';
// import home from './pages/index.vue';
// import aboute from '../deps/node_modules/my-ui/index.vue'

import './test';

const activeName = ref('test')
</script>

<style lang="scss" scoped>
body {
  // not work
  // background: red;
}
</style>
