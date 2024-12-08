<template>
  <el-button type="primary">to about page</el-button>
  <el-alert style="margin-top: 30px;" title="About Page" type="success" description="This is the about page."
    show-icon />

  <component :is="alert" />
</template>

<script lang="tsx" setup>
import { ElAlert } from 'element-plus';
import 'element-plus/theme-chalk/el-alert.css'
import 'element-plus/theme-chalk/el-button.css'

const alert = <ElAlert style="margin-top: 30px;" title="About Page" type="success" description="This is jsx component." show-icon />;
</script>