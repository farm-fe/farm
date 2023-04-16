import { createApp } from 'vue';
import ElementPlus from 'element-plus';
import 'element-plus/dist/index.css';
import Main from './Main.vue';
import './index.css';

createApp(Main).use(ElementPlus).mount('#app');
