<template>
  <vue-monaco-diff-editor
    theme="vs-dark"
    :original="original"
    :modified="modified"
    language="javascript"
    :options="MONACO_EDITOR_OPTIONS"
    @mount="handleMount"
  />
</template>

<script lang="ts">
import { defineComponent, shallowRef } from 'vue';

export default defineComponent({
  name: 'CodeDiff',
  props: {
    original: String,
    modified: String,
    language: String
  },
  setup() {
    const MONACO_EDITOR_OPTIONS = {
      automaticLayout: true,
      formatOnType: true,
      formatOnPaste: true,
      readOnly: true
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
