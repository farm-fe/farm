import {defineDocsConfig} from '@farmfe/docs';
import sass from '@farmfe/plugin-sass';
import tailwindcss from '@farmfe/plugin-tailwindcss';

export default defineDocsConfig({
  title:'Orchard Field Notes',url:'https://orchard.example',basePath:'/manual/',
  locales:{default:'en',items:['en','zh']},
  docs:{collections:[
    {id:'en-current',group:'handbook',dir:'content/en-current',locale:'en',version:'current',routeBase:'/docs/',sidebar:'sidebar.json'},
    {id:'zh-current',group:'handbook',dir:'content/zh-current',locale:'zh',version:'current',routeBase:'/zh/docs/',sidebar:'sidebar.json'},
    {id:'en-old',group:'handbook',dir:'content/en-old',locale:'en',version:'previous',routeBase:'/v1/docs/',sidebar:'sidebar.json'},
    {id:'zh-old',group:'handbook',dir:'content/zh-old',locale:'zh',version:'previous',routeBase:'/zh/v1/docs/',sidebar:'sidebar.json'}
  ]},
  blog:{collections:[{id:'news',dir:'blog',routeBase:'/blog/',blogListContent:'full'}]},
  theme:{
    components:'src/theme.tsx',customCss:['src/orchard.scss','src/tailwind.css'],
    navbar:[{label:'Handbook',to:'/manual/docs/'},{label:'Notes',to:'/manual/blog/'}]
  }
},{
  plugins:[sass(),tailwindcss({config:{corePlugins:{preflight:false}}})],
  compilation:{output:{path:'build'},persistentCache:false,sourcemap:false}
});
