<template>
  <Card title="Plugin Analysis">
    <Table :loading="loading" :columns="columns" :dataSource="tableData"></Table>
  </Card>
</template>

<script lang="ts" setup>
import { computed, ref, onMounted } from 'vue';
import { Card, Table } from 'ant-design-vue';
import { getPluginStats } from '../../api';

interface TableDataType {
  plugin_name: string;
  hook: string;
  calls: number;
  duration: number;
}

const plugin_stats = ref<any | null>(null);
const loading = ref(false);

onMounted(() => {
  loading.value = true;
  getPluginStats().then((data) => {
    // TODO support HMR compilation compare
    plugin_stats.value = JSON.parse(data).initialCompilationFlowStats.hookStatsMap;
    console.log('plugin_stats:', plugin_stats.value);
  }).finally(() => {
    loading.value = false;
  });
});
const tableData = computed(() => {
  if (plugin_stats.value) {
    loading.value = true;
    const map: Record<string, any> = {};
    
    for (const hookName in plugin_stats.value) {
      console.log('hookName:', hookName);
      const hooks = plugin_stats.value[hookName];
      console.log('hooks:', hooks);

      for (const hook of hooks) {
        const { pluginName, duration } = hook;

        if (!map[pluginName]) {
          map[pluginName] = {};
        }
        if (!map[pluginName][hookName]) {
          map[pluginName][hookName] = {
            callCount: 0,
            totalDuration: 0,
            ...hook
          };
        }

        map[pluginName][hookName].callCount++;
        map[pluginName][hookName].totalDuration += duration;
      }
    }

    const result: TableDataType[] = [];

    for (const pluginName in map) {
      for (const hookName in map[pluginName]) {
        const { callCount, totalDuration } = map[pluginName][hookName];
        result.push({
          plugin_name: pluginName,
          hook: hookName,
          calls: callCount,
          duration: totalDuration
        });
      }
    }

    loading.value = false;
    return result;
  }
  return [];
});

const pluginFilter = computed(() => {
  if (plugin_stats.value) {
    return Object.keys(plugin_stats.value).map((pluginName) => {
      return {
        text: pluginName,
        value: pluginName
      };
    });
  } else {
    return [];
  }
});

const columns = computed(() => {
  return [
    {
      title: 'Plugin Name',
      dataIndex: 'plugin_name',
      key: 'plugin_name',
      filters: pluginFilter.value,
      onFilter: (value: string, record: TableDataType) =>
        record.plugin_name.indexOf(value) === 0
    },
    {
      title: 'Hook',
      dataIndex: 'hook',
      key: 'hook'
    },
    {
      title: 'Calls',
      dataIndex: 'calls',
      key: 'calls',
      sorter: (a: TableDataType, b: TableDataType) => a.calls - b.calls
    },
    {
      title: 'Duration(ms)',
      dataIndex: 'duration',
      key: 'duration',
      sorter: (a: TableDataType, b: TableDataType) =>
        a.duration - b.duration
    }
  ];
});
</script>
