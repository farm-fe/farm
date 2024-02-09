<script setup lang="ts">
import { ref } from 'vue';
import Intro from './async-comp';

defineProps<{ msg: string }>()
/**
 * @license MIT brightwu 2024
 */
const count = ref(0);
console.log('current count is', count.value);

interface Tree {
  label: string
  children?: Tree[]
}

const handleNodeClick = (data: Tree) => {
  console.log(data)
}

const data: Tree[] = [
  {
    label: 'Level one 1',
    children: [
      {
        label: 'Level two 1-1',
        children: [
          {
            label: 'Level three 1-1-1',
          },
        ],
      },
    ],
  },
  {
    label: 'Level one 2',
    children: [
      {
        label: 'Level two 2-1',
        children: [
          {
            label: 'Level three 2-1-1',
          },
        ],
      },
      {
        label: 'Level two 2-2',
        children: [
          {
            label: 'Level three 2-2-1',
          },
        ],
      },
    ],
  },
  {
    label: 'Level one 3',
    children: [
      {
        label: 'Level two 3-1',
        children: [
          {
            label: 'Level three 3-1-1',
          },
        ],
      },
      {
        label: 'Level two 3-2',
        children: [
          {
            label: 'Level three 3-2-1',
          },
        ],
      },
    ],
  },
]

const defaultProps = {
  children: 'children',
  label: 'label',
}
</script>

<template>
  <h1>{{ msg }}</h1>

  <div class="card">
    <el-button type="primary" @click="count++">count is {{ count }}</el-button>

    <el-tree :data="data" :props="defaultProps" @node-click="handleNodeClick" />
    <el-date-picker />
    <p>
      Edit
      <code>components/HelloWorld.vue</code> to <span class="read-the-docs">test</span> HMR
    </p>
  </div>

  <intro />
</template>

<style scoped lang="scss">
@forward 'element-plus/theme-chalk/src/mixins/config.scss';
@import 'element-plus/theme-chalk/src/upload.scss';

.read-the-docs {
  color: #888;
}
</style>
