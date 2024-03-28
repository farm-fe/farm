// register vue composition api globally
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from './router';
import { install as VueMonacoEditorPlugin } from '@guolao/vue-monaco-editor';

import 'ant-design-vue/dist/reset.css';
import './style.css';

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);
app.use(VueMonacoEditorPlugin, {
  paths: {
    // The recommended CDN config
    vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.43.0/min/vs'
  }
});

app.use(router);
app.mount('#app');
