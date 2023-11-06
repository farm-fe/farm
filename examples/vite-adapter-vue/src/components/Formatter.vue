<template>
  <div>
    Input Code: <el-input v-model="code" type="textarea" :rows="10" />
  </div>
  <div>
    <el-button type="primary" @click="format">Format</el-button>
  </div>
  <div>original: {{ str }}, bcryptjs: {{ res }}</div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import * as prettier from "prettier";
import bcrypt from "bcryptjs";
console.log(bcrypt);
const code = ref(`const a = 1;`);
const str = ref(`213`);
var salt = bcrypt.genSaltSync(10);
var hash = bcrypt.hashSync("B4c0/\/", salt);
const res = ref(hash);
const format = async () => {
  const formatted = await prettier.format(code.value, {
    parser: "babel",
    plugins: [require("prettier/plugins/babel"), require("prettier/plugins/estree")],
  });
  code.value = formatted;
};
</script>

<style scoped></style>