<template>
  <Card title="Plugin Analysis">
    <div>
      <Select
        style="width: 100%"
       :options="allStats.map((item: any) => ({label:item.entries.join(','), value:item.entries.join(',')}))" v-model:value="selectedStat"></Select>
    </div>
    <Table :loading="loading" :columns="columns" :dataSource="tableData"></Table>
  </Card>
</template>

<script lang="ts" setup>
import { computed, ref, onMounted, watchEffect } from 'vue';
import { Card, Select, Table } from 'ant-design-vue';
import { getPluginStats } from '../../api';

interface TableDataType {
  plugin_name: string;
  hook: string;
  calls: number;
  duration: number;
}

interface HookType {
  hookName: string;
  pluginName: string;
  duration: number;
}

const plugin_stats = ref<any | null>(null);
const allStats = ref<any | null>([]);
const loading = ref(false);

const selectedStat = ref('');

watchEffect(() => {
  if (selectedStat.value) {
    const stat = allStats.value.find((item: any) => item.entries.join(',') === selectedStat.value);
    if (stat) {
      plugin_stats.value = stat.hookStatsMap;
    } else {
      plugin_stats.value = null;
    }
  }
});

onMounted(() => {
  loading.value = true;
  getPluginStats().then((data) => {
    const parsedData = JSON.parse(data);
    const result = [...parsedData.hmrCompilationFlowStats, parsedData.initialCompilationFlowStats];
    allStats.value = result.map((item: any) => {
      item.entries = item.entries.filter((item: any) => !item.endsWith('.farm-runtime'))
      return item;
    });
    // TODO support HMR compilation compare
    plugin_stats.value = parsedData.initialCompilationFlowStats.hookStatsMap;
    selectedStat.value = result[0]?.entries.join(',');
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
      const hooks = plugin_stats.value[hookName] as Array<HookType>;

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

const pluginNameFilter = computed(() => {
  if (plugin_stats.value) {
    const pluginNamesSet = new Set<string>();
    for (const hooks of Object.values(plugin_stats.value)) {
      for (const hook of hooks as Array<HookType>) {
        pluginNamesSet.add(hook.pluginName);
      }
    }
    return Array.from(pluginNamesSet).map((pluginName) => {
      return {
        text: pluginName,
        value: pluginName
      };
    });
  } else {
    return [];
  }
});

const hookFilter = computed(() => {
  if (plugin_stats.value) {
    const hooksSet = new Set<string>();
    for (const hooks of Object.values(plugin_stats.value)) {
      for (const hook of hooks as Array<HookType>) {
        hooksSet.add(hook.hookName);
      }
    }
    return Array.from(hooksSet).map((hookName) => {
      return {
        text: hookName,
        value: hookName
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
      filters: pluginNameFilter.value,
      onFilter: (value: string, record: TableDataType) =>
        record.plugin_name.indexOf(value) === 0
    },
    {
      title: 'Hook',
      dataIndex: 'hook',
      key: 'hook',
      filters: hookFilter.value,
      onFilter: (value: string, record: TableDataType) =>
        record.hook.indexOf(value) === 0,
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