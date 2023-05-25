import { createApp } from 'vue';
import Main from './main.vue';
import Antd from 'ant-design-vue';
import 'ant-design-vue/dist/antd.css';

import router from './router';

createApp(Main).use(router).use(Antd).mount('#app');
