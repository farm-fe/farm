<template>
  <div>
    <Card title="Project Overview">
      <p class="text-gray-400">cpu: <span class="text-gray-950">{{ envInfo?.System?.CPU || "Not Found" }}</span></p>
      <p class="text-gray-400">memory: <span class="text-gray-950">{{ envInfo?.System?.Memory || "Not Found" }}</span>
      </p>
      <p class="text-gray-400">os: <span class="text-gray-950">{{ envInfo?.System?.OS || "Not Found" }}</span></p>
      <p class="text-gray-400">node: <span class="text-gray-950">{{ envInfo?.Binaries?.Node?.version || "Not Found"
          }}</span></p>
      <p class="text-gray-400">npm: <span class="text-gray-950">{{ envInfo?.Binaries?.npm?.version || "Not Found"
          }}</span></p>
      <p class="text-gray-400">yarn: <span class="text-gray-950">{{ envInfo?.Binaries?.Yarn?.version || "Not Found"
          }}</span></p>
      <p class="text-gray-400">pnpm: <span class="text-gray-950">{{ envInfo?.Binaries?.pnpm?.version || "Not Found"
          }}</span></p>
      <p class="text-gray-400">farm: <span class="text-gray-950">{{ typeof envInfo?.npmPackages["farm"] === "object" ?
        envInfo?.npmPackages["farm"].installed : "Not Found" }}</span></p>
    </Card>
  </div>
</template>


<script lang="ts">
import { defineComponent, ref } from 'vue';
import { Card } from 'ant-design-vue';
import { getFarmEnvInfo } from '../api';
import { FarmEnvInfo } from '../../../node/utils/envinfo';

export default defineComponent({
  name: 'Home',
  components: {
    Card
  },
  setup() {
    const envInfo = ref<FarmEnvInfo | undefined>(undefined);
    getFarmEnvInfo().then((res) => {
      envInfo.value = res;
    });
    return { envInfo };
  }
});
</script>
