<template>
  <el-tabs v-model="activeName">
    <img src="/logo.png" alt="">
    <el-tab-pane label="test" name="test">
      Map is weakMap: {{ mapIsWeakMap }};
      WeakMap is weakMap: {{ weakMapIsWeakMap }};

      <ElButton type="primary" @click="hashValue">
        Argon2 hash
      </ElButton>
      Hash Bytes: {{ hashBytes }}
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
import { ElTabs, ElTabPane, ElButton } from 'element-plus'
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
import { isWeakMap } from 'lodash-es';

const hashBytes = ref('')

async function hashValue() {
  const { hash: getHash, ArgonType } = await import('argon2-browser/dist/argon2-bundled.min')
  const { hash } = await getHash({
    pass: 'any string here',
    salt: Date.now().toString(),
    time: 10,
    mem: 8 * 2 * 10,
    hashLen: 32,
    type: ArgonType.Argon2id
  })
  hashBytes.value = hash
}


const activeName = ref('test')
console.log(activeName)
const mapIsWeakMap = isWeakMap(new Map())
const weakMapIsWeakMap = isWeakMap(new WeakMap())
</script>

<style lang="scss" scoped>
body {
  // not work
  // background: red;
}
</style>
