import { createApp } from 'vue';
import Main from './main.vue';
import Antd from 'ant-design-vue';
import 'ant-design-vue/dist/antd.css';

createApp(Main).use(Antd).mount('#app');
