<template>
  <div>
    <Title></Title>
    <Description></Description>
    <ElButton>123132</ElButton>
    <el-date-picker v-model="value1" type="date" placeholder="Pick a day" />
  </div>
</template>

<script lang="ts">
import { ElButton } from 'element-plus';
import { defineComponent, ref } from 'vue';
import Title from './components/Title.vue';
import Description from './components/Description.vue';

export default defineComponent({
  setup() {
    const value1 = ref('');
    return {
      value1,
    };
  },
  components: {
    Title,
    Description,
    ElButton,
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
