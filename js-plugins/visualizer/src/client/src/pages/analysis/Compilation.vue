<template>
  <Layout class="bg-transparent">
    <Layout.Sider theme="light" width="25%">
      <Card title="Compilation History">
        <template #extra> <Button @click="fetchStats">Refresh</Button></template>

        <List :loading="loading">
          <List.Item :class="['cursor-pointer', {
            'bg-gray-200 border-gray-300': item === currentSelectedStats
          }]" v-for="item in allAllStats" :key="item.startTime" @click="() => currentSelectedStats = item">
            <div class="w-full">
              <Popover :content="item.entries.join(',')">
                <div class="font-bold text-sm text-ellipsis overflow-hidden whitespace-nowrap w-full">
                  {{ item.entries.join(',') }}
                </div>
              </Popover>
              <div class="flex items-center mt-2">
                <Tag color="green">{{ item.isFirstCompilation ? 'Initial Compilation' : 'HMR' }}</Tag>
                <Tag color="blue">{{ item.duration }}ms</Tag>
                <Tag color="yellow">{{ new Date(item.startTime).toLocaleString() }}</Tag>
              </div>
            </div>
          </List.Item>
        </List>
      </Card>
    </Layout.Sider>

    <Layout.Content class="min-h-[400px] ml-2">
      <div v-if="currentSelectedStats">
        <Card title="Compilation Details">
          <div>
            <span class="font-bold mb-2">Select Module</span>
            <Select style="width: 70%; margin-left: 15px;" mode="multiple" v-model:value="selectedFilteredModules" @search="searchFilteredModules">
              <Select.Option v-for="m in filteredModules" :key="m.moduleId" :label="m.moduleId">
                <span>{{ m.moduleId }}</span>
                <Tag :color="m.moduleType === 'css' ? 'purple' : 'blue'" class="ml-2">{{ m.moduleType }}</Tag>
              </Select.Option>
            </Select>

            <Button class="ml-2" @click="clearAllSelection">Clear All Selection</Button>
          </div>

          <Tabs v-model:active-key="activeKey">
            <Tabs.TabPane v-for="hook in allSupportedHooks" :key="hook" :tab="hook">
              <div class="flex justify-between mb-2 items-start">
                <Card class="w-[300px]" v-if="getModules(hook).length">
                  <InputSearch placeholder="Input and Search Module" @change="val => searchModule(hook, [val.target.value])" />
                  
                  <List class="max-h-[600px] mt-2">
                    <List.Item :key="module" v-for="module in modules" :class="['cursor-pointer', '!pl-1', '!pr-1', {
                      'bg-gray-200 border-gray-300': module === currentSelectedModule
                    }]" @click="() => currentSelectedModule = module">
                      <Popover :content="module">
                        <div class="text-sm text-ellipsis overflow-hidden whitespace-nowrap w-full font-medium">
                          {{ module }}
                        </div>
                      </Popover>
                    </List.Item>
                  </List>
                </Card>
                <Card class="w-[200px] ml-2">
                  <List>
                    <List.Item :key="plugin" v-for="plugin in plugins" :class="['cursor-pointer', '!pl-1', '!pr-1', {
                      'bg-gray-200 border-gray-300': plugin === currentSelectedPlugin
                    }]" @click="() => currentSelectedPlugin = plugin">
                      <Popover :content="plugin">
                        <div class="text-sm text-ellipsis overflow-hidden whitespace-nowrap w-full">
                           {{ plugin }}
                        </div>
                      </Popover>
                    </List.Item>
                  </List>
                </Card>
                <Card class="w-full ml-2 flex-1">
                  <Select mode="multiple" style="width: 400px" v-model:value="hookDetails" placeholder="Input and Search Hook Detail" @search="val => searchHooks(hook, [val])">
                    <Select.Option v-for="(_, k) in filteredHooksMap" :value="k" :key="k">
                      {{ k }}
                    </Select.Option>
                  </Select>

                  <div v-if="hookDetails?.length" class="mt-4">
                    <Descriptions v-for="item in hookDetails.map(k => filteredHooksMap[k])" :key="item.input" class="border-gray-100 border border-l-0 border-r-0 p-2">
                      <DescriptionsItem label="pluginName" :span="2"><Tag>{{ item.pluginName }}</Tag></DescriptionsItem>
                      <DescriptionsItem label="hookName"><Tag>{{ item.hookName }}</Tag></DescriptionsItem>
                      <DescriptionsItem label="input/output diff" :span="2">
                        <Button @click="() => setDiffInfo(item)">View Diff</Button>
                        <Button @click="() => setViewInfo(item)" class="ml-2">View Output</Button>
                      </DescriptionsItem>
                      <DescriptionsItem label="duration"><Tag color="green">{{ item.duration }}ms</Tag></DescriptionsItem>
                      <DescriptionsItem label="startTime"><Tag color="yellow">{{ new Date(item.startTime).toLocaleString() }}</Tag></DescriptionsItem>
                      <DescriptionsItem label="endTime"><Tag color="yellow">{{ new Date(item.endTime).toLocaleString() }}</Tag></DescriptionsItem>
                    </Descriptions>
                  </div>
                  <div v-else class="mt-4">Please select module and plugin first</div>
                </Card>
              </div>
            </Tabs.TabPane>
          </Tabs>
        </Card>
      </div>
      <div v-else><Spin><Empty description="Stats is loading, please wait a few seconds..." /></Spin></div>
    </Layout.Content>
  </Layout>

  <Drawer v-model:open="isDiffOpen" :title="`${diffInfo?.moduleId} - ${diffInfo?.pluginName} - ${diffInfo?.hookName}`" placement="right" width="80vw">
    <CodeDiff :original="diffInfo?.input" :modified="diffInfo?.output" :language="'javascript'"></CodeDiff>
  </Drawer>

  <Drawer
      v-model:open="isViewOpen"
      :title="viewInfo?.moduleId + ' - ' + viewInfo?.pluginName + ' - ' + viewInfo?.hookName"
      placement="right"
      width="80vw"
    >
      <CodeViewer :code="viewInfo?.content" :language="'javascript'"></CodeViewer>
    </Drawer>
</template>

<script lang="ts" setup>
import { ref, onMounted, watchEffect, watch } from 'vue';
import { Layout, Card, List, Tag, Empty, Tabs, Popover, InputSearch, Select, Button, Descriptions, DescriptionsItem, Drawer, Spin } from 'ant-design-vue';
import { getPluginStats } from '../../api';
import CodeDiff from '../../components/CodeDiff.vue';
import CodeViewer from '../../components/CodeViewer.vue';

const allAllStats = ref<any[]>([]);
const currentSelectedStats = ref<any>();
const allSupportedHooks = ref<any[]>([]);
const allModules = ref<any[]>([]);

const filteredModules = ref<any[]>([]);
const selectedFilteredModules = ref<string[]>([]);

const activeKey = ref<string>('');

const modules = ref<any[]>([]);
const currentSelectedModule = ref<any>();

const plugins = ref<any[]>([]);
const currentSelectedPlugin = ref<any>();

const filteredHooksMap = ref<any>({});
const hookDetails = ref<any[]>();

const diffInfo = ref<any>();
const viewInfo = ref<any>();
const isDiffOpen = ref<boolean>(false);
const isViewOpen = ref<boolean>(false);

const setDiffInfo = (h: any) => {
  diffInfo.value = {
    moduleId: h.moduleId,
    pluginName: h.pluginName,
    hookName: h.hookName,
  };
  if (h.hookName === 'transform') {
    const input = JSON.parse(h.input || '{}');
    const output = JSON.parse(h.output || '{}');
    diffInfo.value.input = input.content;
    diffInfo.value.output = output.content;
  } else if (h.hookName === 'generate_resources') {
    const input = JSON.parse(h.input || '{}');
    const output = JSON.parse(h.output || '{}');
    diffInfo.value.input = JSON.stringify(input, null, 2);
    const string = new TextDecoder().decode(new Uint8Array(output.resource.bytes));
    diffInfo.value.output = string;
  } else {
    const input = JSON.parse(h.input || '{}');
    const output = JSON.parse(h.output || '{}');
    diffInfo.value.input = JSON.stringify(input, null, 2);
    diffInfo.value.output = JSON.stringify(output, null, 2);
  }
  isDiffOpen.value = true;
}

const setViewInfo = (h: any) => {
  viewInfo.value = {
    moduleId: h.moduleId,
    pluginName: h.pluginName,
    hookName: h.hookName,
  };
  const output = JSON.parse(h.output || '{}');
  if (output.content || output.renderedContent) {
    viewInfo.value.content = output.content || output.renderedContent;
  } else if (output.resource?.bytes) {
    // utf-8 bytes to string
    const string = new TextDecoder().decode(new Uint8Array(output.resource.bytes));
    viewInfo.value.content = string;
  } else {
    viewInfo.value.content = JSON.stringify(output, null, 2);
  }
  isViewOpen.value = true;
}

const getDetailKey = (h: any) => {
  const result = `${h.input}${h.output}`;
  if (!result) {
    return JSON.stringify(h);
  }

  return result.slice(0, 100) + result.slice(-100);
};

const getModules = (hookName: string) => {
  if (!currentSelectedStats.value) {
    return [];
  }
  const result: string[] = [];
  currentSelectedStats.value.hookStatsMap[hookName]?.forEach((item: any) => {
    if (!currentSelectedModule.value && currentSelectedPlugin.value && item.pluginName !== currentSelectedPlugin.value) {
      return;
    }
    if (item.moduleId) {
      result.push(item.moduleId);
    }
  });
  console.log('getModules:', result);
  return [...new Set(result)];
}

const searchModule = (hookName: string, value: (string|undefined)[] = []) => {
  console.log('searchModule:', value);
  modules.value = getModules(hookName).filter((item: any) => {
    if (!value?.length) {
      return true;
    }

    if (Array.isArray(value)) {
      return value.some((v: any) => item.includes(v));
    }

    return item.includes(value);
  });
}

const getPlugins = (hookName: string) => {
  if (!currentSelectedStats.value) {
    return [];
  }

  const result: string[] = [];
  currentSelectedStats.value.hookStatsMap[hookName]?.forEach((item: any) => {
    if (currentSelectedModule.value && item.moduleId === currentSelectedModule.value) {
      result.push(item.pluginName);
    } else if (selectedFilteredModules.value?.length) {
      if (selectedFilteredModules.value.includes(item.moduleId)) {
        result.push(item.pluginName);
      }
    } else if (!currentSelectedModule.value) {
      result.push(item.pluginName);
    }
  });

  return [...new Set(result)];
}

const searchPlugins = (hookName: string, value = '') => {
  plugins.value = getPlugins(hookName).filter((item: any) => {
    if (!value) {
      return true;
    }

    return item.includes(value);
  });
  console.log('searchPlugins:', value, plugins.value);
}

const searchFilteredModules = (value = '') => {
  filteredModules.value = allModules.value.filter((item: any) => {
    if (!value) {
      return true;
    }

    return item.moduleId.includes(value);
  });
}

const getAllSupportedHooks = (moduleIds: string[] = []) => {
  if (!currentSelectedStats.value) {
    return [];
  }

  const hooks: any[] = [];

  Object.values(currentSelectedStats.value.hookStatsMap).forEach((hook: any) => {
    if (moduleIds.length && !hook.some((item: any) => moduleIds.includes(item.moduleId))) {
      return;
    }


    if (hook[0] && !hooks.includes(hook?.[0].hookName)) {
      hooks.push(hook?.[0].hookName);
    }
  })

  // sort the hooks by resolve -> load -> transform ...
  const orderedHooks = [
    'update_modules', 'resolve', 'load', 'transform', 'parse', 'process_module', 'analyze_deps',
    'finalize_module', 'optimize_module_graph', 'partial_bundling', 'render_resource_pot_modules', 
    'render_resource_pot', 'generate_resources'
  ];
  let inOrderedHooks = hooks.filter((item: any) => {
    return orderedHooks.includes(item);
  });
  let unOrderedHooks = hooks.filter((item: any) => {
    return !orderedHooks.includes(item);
  });
  inOrderedHooks = inOrderedHooks.sort((a, b) => {
    return orderedHooks.indexOf(a) - orderedHooks.indexOf(b);
  });
  return [...inOrderedHooks, ...unOrderedHooks];
}

const searchHooks = (hookName: string, value: any[] = []) => {
  if (!currentSelectedStats.value) {
    return;
  }

  const hooks = currentSelectedStats.value.hookStatsMap[hookName]?.filter((item: any) => {
    if (currentSelectedModule.value && item.moduleId !== currentSelectedModule.value) {
      return false;
    }

    if (currentSelectedPlugin.value && item.pluginName !== currentSelectedPlugin.value) {
      return false;
    }

    if (selectedFilteredModules.value?.length) {
      if (!selectedFilteredModules.value.includes(item.moduleId)) {
        return false;
      }
    }

    if (!value?.length) {
      return true;
    }

    if (Array.isArray(value)) {
      return value.some((v: any) => getDetailKey(item).includes(v));
    }

    return item.includes(value);
  }) ?? [];
  filteredHooksMap.value = hooks.reduce((acc: any, cur: any) => {
    acc[getDetailKey(cur)] = cur;
    return acc;
  }, {});

  console.log('searchHooks:', value, filteredHooksMap.value);
}

const clearAllSelection = () => {
  selectedFilteredModules.value = [];
  currentSelectedModule.value = undefined;
  currentSelectedPlugin.value = undefined;
  hookDetails.value = [];
}

const loading = ref<boolean>(false);
const fetchStats = () => {
  loading.value = true;
  getPluginStats().then((data: any) => {
    data = JSON.parse(data);
    console.log('data:', data);
    data.hmrCompilationFlowStats ??= [];
    data.hmrCompilationFlowStats?.reverse();
    data.initialCompilationFlowStats.isFirstCompilation = true;

    const result = [...data.hmrCompilationFlowStats, data.initialCompilationFlowStats];
    allAllStats.value = result.map((item: any) => {
      item.entries = item.entries.filter((item: any) => !item.endsWith('.farm-runtime'))
      return item;
    });
    currentSelectedStats.value = result[0];
    console.log('allAllStats:', allAllStats.value);
  }).finally(() => {
    loading.value = false;
  });
};

onMounted(() => {
  fetchStats();
});

watchEffect(() => {
  // filter modules and plugins
  hookDetails.value = undefined;
  console.log('selectedFilteredModule:', selectedFilteredModules.value);
  allSupportedHooks.value = getAllSupportedHooks(selectedFilteredModules.value);
  if (!activeKey.value || !allSupportedHooks.value?.includes(activeKey.value)) {
    activeKey.value = allSupportedHooks.value?.[0];
  }
  searchModule(activeKey.value, selectedFilteredModules.value);
  searchPlugins(activeKey.value);
  searchHooks(activeKey.value);
});

watch(() => currentSelectedStats.value, (val) => {
  if (!val) {
    return;
  }
  allModules.value = Object.values(val.moduleGraphStats.modules);
  filteredModules.value = allModules.value;
  console.log('allModules:', allModules.value, val);
});
</script>
