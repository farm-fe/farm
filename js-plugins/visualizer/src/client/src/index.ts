import './style.css';
import 'ant-design-vue/dist/reset.css';

import { install as VueMonacoEditorPlugin } from '@guolao/vue-monaco-editor';
import {
  conf as javascriptConf,
  language as javascriptLanguage
} from 'monaco-editor/esm/vs/basic-languages/javascript/javascript';
import {
  conf as typescriptConf,
  language as typescriptLanguage
} from 'monaco-editor/esm/vs/basic-languages/typescript/typescript';
import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
import 'monaco-editor/esm/vs/language/css/monaco.contribution';
import 'monaco-editor/esm/vs/language/html/monaco.contribution';
import 'monaco-editor/esm/vs/language/json/monaco.contribution';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker';
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker';
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
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
      return new editorWorker();
    }
    return new editorWorker();
  }
};

function registerStandaloneLanguage(
  id: string,
  aliases: string[],
  extensions: string[],
  conf: Parameters<typeof monaco.languages.setLanguageConfiguration>[1],
  language: Parameters<typeof monaco.languages.setMonarchTokensProvider>[1]
) {
  if (!monaco.languages.getLanguages().some((item) => item.id === id)) {
    monaco.languages.register({ id, aliases, extensions });
  }

  monaco.languages.setLanguageConfiguration(id, conf);
  monaco.languages.setMonarchTokensProvider(id, language);
}

registerStandaloneLanguage(
  'javascript',
  ['JavaScript', 'javascript', 'js'],
  ['.js', '.jsx', '.mjs', '.cjs'],
  javascriptConf,
  javascriptLanguage
);

registerStandaloneLanguage(
  'typescript',
  ['TypeScript', 'typescript', 'ts'],
  ['.ts', '.tsx', '.mts', '.cts'],
  typescriptConf,
  typescriptLanguage
);

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);
app.use(VueMonacoEditorPlugin, {
  monaco
});

app.use(router);
app.mount('#app');
