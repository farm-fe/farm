<template>
  <Card title="Resource Pots" class="overflow-hidden"
    :bodyStyle="{ overflow: 'scroll', maxHeight: '500px', padding: '10px 0px' }">
    <div class="flex flex-col">
      <div v-for="item in pots" :key="item.name" class="flex items-center mb-2 pl-6 py-2 border-b border-gray-200"
        @click="selectResourcePot(item)">
        <span class="mr-2">{{ item.name }}</span>
        <Tag color="green">{{ formatSize(item.bytes.length) }}</Tag>
        <Tag v-if="item.info?.data.isEntry" color="blue">entry</Tag>
        <Button size="small" class="flex justify-center items-center" @click="viewSourceCode(item)">
          <CodepenCircleFilled />
        </Button>
      </div>
    </div>
  </Card>
</template>

<script lang="ts">
import { Resource } from 'farm';
import { PropType, defineComponent } from 'vue';
import { Card, Tag, Button } from 'ant-design-vue';
import { CodepenCircleFilled } from '@ant-design/icons-vue';
import { formatSize } from '../utils/size';
import { getResource } from '../api';

import { useResourcePotStore } from '../stores/resourcePot';

export default defineComponent({
  name: 'ResourcePots',
  components: {
    Card,
    Tag,
    Button,
    CodepenCircleFilled
  },
  props: {
    pots: {
      type: Array as PropType<Resource[]>
    }
  },
  setup(_, { emit }) {
    const resourcePotStore = useResourcePotStore();

    function viewSourceCode(resource: Resource) {
      getResource(resource.name).then((data) => {
        emit('view', {
          name: resource.name,
          code: data,
          language: resource.resourceType
        });
      });
    }

    function selectResourcePot(resource: Resource) {
      resourcePotStore.setResource(resource);
      emit('select', resource);
    }

    return { formatSize, viewSourceCode, selectResourcePot };
  }
});
</script>
