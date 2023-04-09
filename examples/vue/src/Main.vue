<template>
  <div>
    <el-button>我是 ElButton</el-button>
    <Title></Title>
    <Description></Description>
  </div>
</template>

<script lang="ts">
import { ElButton } from 'element-plus';
import { defineComponent } from 'vue';
import Title from './components/Title.vue';
import Description from './components/Description.vue';

export default defineComponent({
  setup() {},
  components: {
    Title,
    Description,
    ElButton
  },
});
</script>

<style lang="scss">
$font-size: 24px;
#app {
  font-size: $font-size;
  /* color: red; */
  /* display: flex; */
  /* flex-flow: row wrap; */
  /* justify-content: center; */
  /* align-items: center; */
  height: 300px;
  text-align: center;
}
</style>
