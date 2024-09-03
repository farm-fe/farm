<template>
  <Card
    title="Module List"
    class="overflow-hidden"
    :bodyStyle="{ overflow: 'scroll', maxHeight: '500px', padding: '10px 0px' }"
  >
    <div class="flex flex-col">
      <div
        v-for="item in filterList"
        :key="item.id"
        class="flex flex-col mb-2 pl-6 py-2 border-b border-gray-200 cursor-pointer"
        :class="{ 'text-purple-500': current === item.id }"
        @click="selectModule(item)"
      >
        <div class="text-sm font-bold">{{ item.id }}</div>
        <div class="mt-2 flex items-center">
          <Tag color="green">{{ formatSize(item.size) }}</Tag>
          <Tag color="blue">{{item.moduleType}}</Tag>
          <Tag v-if="item.immutable" color="red">immutable</Tag>
        </div>
      </div>
    </div>
  </Card>
</template>

<script lang="ts">
import { ref, defineComponent, computed } from 'vue';
import { Card, Tag, Button } from 'ant-design-vue';
import { CodepenCircleFilled } from '@ant-design/icons-vue';
import { formatSize } from '../utils/size';
import { getModules } from '../api';
import { Module } from '@farmfe/core/binding/binding';

export default defineComponent({
  name: 'ResourcePots',
  components: {
    Card,
    Tag,
    Button,
    CodepenCircleFilled
  },
  setup(_, { emit }) {
    const moduleList = ref<Module[]>([]);
    const current = ref<string>('');
    const filterList = computed(() => {
      return moduleList.value.filter((item) => !item.immutable);
    });

    getModules().then((res) => {
      moduleList.value = res;
    });

    function selectModule(module: Module) {
      current.value = module.id;
      emit('select', module);
    }

    return { current, formatSize, selectModule, moduleList, filterList };
  }
});
</script>
