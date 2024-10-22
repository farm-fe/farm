
<template>
  <div class="tableau" ref="tableau" v-html="html"></div>
  <button @click="exportFile">Export2222 XLSX</button>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { read, utils, writeFileXLSX } from 'xlsx-js-style';
import { Button } from 'ant-design-vue';

const html = ref("");
const tableau = ref();

onMounted(async () => {
  /* Download from https://sheetjs.com/pres.numbers */
  const f = await fetch("https://sheetjs.com/pres.numbers");
  const ab = await f.arrayBuffer();

  /* parse workbook */
  const wb = read(ab);

  /* update data */
  html.value = utils.sheet_to_html(wb.Sheets[wb.SheetNames[0]]);
});

/* get live table and export to XLSX */
function exportFile() {
  const wb = utils.table_to_book(tableau.value.getElementsByTagName("TABLE")[0])
  writeFileXLSX(wb, "SheetJSVueHTML.xlsx");
}
</script>

<style scoped lang="less">
.tableau {
  margin: 1em;
  border: 1px solid #ccc;
  padding: 1em;
}
</style>
