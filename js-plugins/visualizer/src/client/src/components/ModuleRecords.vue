<template>
  <Card
    :title="moduleId? `${moduleId}` : 'Module Records'"
    class="overflow-hidden"
    :bodyStyle="{ flex: '1', overflow: 'hidden' }"
  >
    <Tabs v-model:activeKey="activeKey" class="h-full">
      <TabPane key="resolve" tab="Resolve Record">
        <ResolveRecord :module-id="moduleId"></ResolveRecord>
      </TabPane>

      <TabPane key="transform" tab="Transform Record">
        <TransformRecord :module-id="moduleId"></TransformRecord>
      </TabPane>

      <TabPane key="process" tab="Process Record">
        <ProcessRecord :module-id="moduleId"></ProcessRecord>
      </TabPane>

      <TabPane key="analyze-deps" tab="AnalyzeDeps Record">
        <AnalyzeDepsRecord :module-id="moduleId"></AnalyzeDepsRecord>
      </TabPane>
    </Tabs>
  </Card>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { Card, Tag, Button, Tabs, TabPane } from 'ant-design-vue';
import { CodepenCircleFilled } from '@ant-design/icons-vue';
import TransformRecord from './TransformRecord.vue';
import ResolveRecord from './ResolveRecord.vue';
import ProcessRecord from './ProcessRecord.vue';
import AnalyzeDepsRecord from './AnalyzeDepsRecord.vue';

export default defineComponent({
  name: 'ModuleRecords',
  components: {
    Card,
    Tag,
    Button,
    CodepenCircleFilled,
    Tabs,
    TransformRecord,
    ResolveRecord,
    ProcessRecord,
    AnalyzeDepsRecord,
    TabPane
  },
  props: {
    moduleId: {
      type: String
    }
  },
  setup() {
    const activeKey = ref('resolve');
    return { activeKey };
  }
});
</script>


<style scoped>
:deep(.ant-tabs-content) {
  height: 100%;
}
</style>