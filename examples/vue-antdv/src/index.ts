import { createApp } from 'vue';
import Main from './main.vue';
import Antd from 'ant-design-vue';
import 'ant-design-vue/dist/reset.css';

import router from './router';
import '/@/index.less';

createApp(Main).use(router).use(Antd).mount('#root');
