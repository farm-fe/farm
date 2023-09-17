<template>
  <div>
    <a-button type="primary" @click="showDrawer">Open</a-button>
  <a-drawer
    v-model:open="open"
    class="custom-class"
    root-class-name="root-class-name"
    :root-style="{ color: 'blue' }"
    style="color: red"
    title="Basic Drawer"
    placement="right"
    @after-open-change="afterOpenChange"
  >
    <p>Some contents...</p>
    <p>Some contents...</p>
    <p>Some contents...</p>
  </a-drawer>
    <RouterView />
  </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import { DatePicker } from 'ant-design-vue';
const open = ref<boolean>(true);

const afterOpenChange = (bool: boolean) => {
  console.log('open', bool);
};

const showDrawer = () => {
  open.value = true;
};
const selectedDate = ref<string>();
</script>