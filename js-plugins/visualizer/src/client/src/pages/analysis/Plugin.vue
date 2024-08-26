<template>
  <Card title="Plugin Analysis">
    <Table :columns="columns" :dataSource="tableData"></Table>
  </Card>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from 'vue';
import { Card, Table } from 'ant-design-vue';
import { getPluginStats } from '../../api';
import { PluginStats } from '@farmfe/core';

interface TableDataType {
  plugin_name: string;
  hook: string;
  calls: number;
  duration: number;
}
export default defineComponent({
  name: 'PluginAnalyze',
  components: {
    Card,
    Table
  },
  setup() {
    const plugin_stats = ref<PluginStats | null>(null);
    getPluginStats().then((data) => {
      plugin_stats.value = JSON.parse(data);
      console.log('plugin_stats:', plugin_stats.value);
    });

    const tableData = computed(() => {
      if (plugin_stats.value) {
        const result = [];
        for (const pluginName in plugin_stats.value) {
          console.log('pluginName:', pluginName);
          const hooks = plugin_stats.value[pluginName];
          console.log('hooks:', hooks);
          for (const hookName in hooks) {
            const { totalDuration, callCount } = hooks[hookName];
            result.push({
              plugin_name: pluginName,
              hook: hookName,
              calls: callCount,
              duration: totalDuration / 1000
            });
          }
        }
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

    return { tableData, columns };
  }
});
</script>
