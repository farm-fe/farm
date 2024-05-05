<template>
  <div class="w-full h-full">
    <vue-monaco-editor
      v-model:value="code"
      theme="vs-dark"
      :options="MONACO_EDITOR_OPTIONS"
      :language="formatLanguage(language)"
      @mount="handleMount"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent, shallowRef } from 'vue';

export default defineComponent({
  name: 'CodeViewer',
  props: {
    code: String,
    language: String
  },
  setup() {
    const MONACO_EDITOR_OPTIONS = {
      automaticLayout: true,
      formatOnType: true,
      formatOnPaste: true
    };

    const editorRef = shallowRef();
    const handleMount = (editor: any) => (editorRef.value = editor);

    function formatLanguage(lang?: string) {
      if (!lang) {
        return 'javascript';
      }
      const data = lang.toLocaleLowerCase();
      if (data === 'js' || data === 'runtime') {
        return 'javascript';
      } else {
        return data;
      }
    }

    return {
      MONACO_EDITOR_OPTIONS,
      handleMount,
      formatLanguage
    };
  }
});
</script>
