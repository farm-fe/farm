<script setup lang="ts">
import { ref } from 'vue';
import dayjs from 'dayjs';
import HelloWorld from './components/HelloWorld.vue';
const initials = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j'];

const options = Array.from({ length: 1000 }).map((_, idx) => ({
  value: `Option ${idx + 1}`,
  label: `${initials[idx % 10]}${idx}`
}));
const value = ref(Date.now() + 1000 * 60 * 60 * 7);
const value1 = ref(Date.now() + 1000 * 60 * 60 * 24 * 2);
const value2 = ref(dayjs().add(1, 'month').startOf('month'));

function reset() {
  value1.value = Date.now() + 1000 * 60 * 60 * 24 * 2;
}
</script>

<template>
  <el-row>
    <el-col :span="8">
      <el-countdown title="Start to grab" :value="value" />
    </el-col>
    <el-col :span="8">
      <el-countdown
        title="Remaining VIP time"
        format="HH:mm:ss"
        :value="value1"
      />
      <el-button class="countdown-footer" type="primary" @click="reset"
        >Reset
      </el-button>
    </el-col>
    <el-col :span="8">
      <el-countdown format="DD [days] HH:mm:ss" :value="value2">
        <template #title>
          <div style="display: inline-flex; align-items: center">
            <el-icon style="margin-right: 4px" :size="12">
              <Calendar />
            </el-icon>
            Still to go until next month
          </div>
        </template>
      </el-countdown>
      <div class="countdown-footer">{{ value2.format('YYYY-MM-DD') }}</div>
    </el-col>
  </el-row>
  <div>
    <a href="https://farm-fe.github.io/" target="_blank">
      <img src="./assets/logo.png" class="logo" alt="Farm logo" />
    </a>
    <a href="https://vuejs.org/" target="_blank">
      <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
    </a>
  </div>

  <el-config-provider :size="'large'" :z-index="3000">
    <HelloWorld msg="Farm + Vue" />
    <el-button type="primary">123123sdsadsad</el-button>
  </el-config-provider>
</template>

<style scoped>
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: filter 300ms;
}

.logo:hover {
  filter: drop-shadow(0 0 2em #9f1a8faa);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>
