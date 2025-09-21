import './style.css';
import 'ant-design-vue/dist/reset.css';

import { install as VueMonacoEditorPlugin } from '@guolao/vue-monaco-editor';
import * as monaco from 'monaco-editor';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker';
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker';
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker';
import { createPinia } from 'pinia';
// register vue composition api globally
import { createApp } from 'vue';
import App from './App.vue';
import router from './router';

// @ts-ignore
self.MonacoEnvironment = {
  getWorker(_: any, label: string) {
    if (label === 'json') {
      return new jsonWorker();
    }
    if (label === 'css' || label === 'scss' || label === 'less') {
      return new cssWorker();
    }
    if (label === 'html' || label === 'handlebars' || label === 'razor') {
      return new htmlWorker();
    }
    if (label === 'typescript' || label === 'javascript') {
      return new tsWorker();
    }
    return new editorWorker();
  }
};

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);
app.use(VueMonacoEditorPlugin, {
  monaco
});

app.use(router);
app.mount('#app');
