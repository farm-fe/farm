<template>
  <div direction="vertical" style="width: 100%; text-align: center">
    <img :src="logo" alt="">
    <h3>Welcome to use Vue2.7!</h3>
    <el-button>默认按钮</el-button>
  <el-button type="primary">主要按钮</el-button>
  <el-button type="success">成功按钮</el-button>
  </div>
</template>

<script>
import { Button } from 'element-ui';
import Logo from './assets/logo.png';
export default {
  components: {
    [Button.name]: Button,
  },
  data() {
    return {
      logo: Logo
    };
  },
};
</script>