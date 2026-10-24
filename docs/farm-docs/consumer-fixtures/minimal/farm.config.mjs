import {defineDocsConfig} from '@farmfe/docs';

export default defineDocsConfig({
  title:'Orchard Handbook',url:'https://orchard.example',basePath:'/',
  docs:{collections:[{id:'guide',dir:'docs',routeBase:'/',sidebar:'sidebar.json'}]}
},{compilation:{output:{path:'build'},persistentCache:false,sourcemap:false}});
