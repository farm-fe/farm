import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const require=createRequire(import.meta.url);
const playwrightPath=process.env.FARM_DIAGRAM_PLAYWRIGHT;
if(!playwrightPath)throw Error('Set FARM_DIAGRAM_PLAYWRIGHT to an installed playwright-core module');
const {chromium}=require(playwrightPath);
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const folder=path.join(root,'website/static/diagrams');
const cli=process.argv[2];
const browser=process.env.FARM_DIAGRAM_CHROMIUM;
if(!cli||!browser)throw Error('Usage: FARM_DIAGRAM_CHROMIUM=<existing Chromium> node scripts/generate-doc-diagrams.mjs <pinned Mermaid CLI src/cli.js>');
const pkg=JSON.parse(await fs.readFile(path.resolve(path.dirname(cli),'../package.json'),'utf8'));
if(pkg.name!=='@mermaid-js/mermaid-cli'||pkg.version!=='12.0.0')throw Error('Use isolated @mermaid-js/mermaid-cli@12.0.0');
const vendorRequire=createRequire(path.resolve(path.dirname(cli),'../package.json'));
const mermaidBundle=path.join(path.dirname(vendorRequire.resolve('mermaid')),'mermaid.min.js');
const mermaidVersion=JSON.parse(await fs.readFile(path.join(path.dirname(mermaidBundle),'../package.json'),'utf8')).version;
if(mermaidVersion!=='12.2.0')throw Error('Use locked mermaid@12.2.0');
const engine=await chromium.launch({headless:true,executablePath:browser});
const page=await engine.newPage();
await page.setContent('<!doctype html><html><body></body></html>');
await page.addScriptTag({path:mermaidBundle});
const hash=x=>createHash('sha256').update(x).digest('hex');
const manifest={schemaVersion:1,renderer:`mermaid@${mermaidVersion}`,authoringPackage:'@mermaid-js/mermaid-cli@12.0.0',driver:'playwright-core',diagrams:[]};
const config={securityLevel:'strict',htmlLabels:false,deterministicIds:true,markdownAutoWrap:false,fontFamily:'system-ui, sans-serif',fontSize:15,flowchart:{htmlLabels:false,nodeSpacing:28,rankSpacing:36,wrappingWidth:480,padding:16,useMaxWidth:false,curve:'linear'}};
try{

 const names=(await fs.readdir(folder)).filter(n=>n.endsWith('.mmd')).sort();
 for(const name of names){
  const source=await fs.readFile(path.join(folder,name));
  if(/\bclick\s|https?:\/\//i.test(source.toString()))throw Error('Diagrams cannot link external resources');
  const entry={source:name,sourceSha256:hash(source),outputs:{}};
  for(const mode of ['light','dark']){
   const tone=mode==='dark'?{bg:'#19171f',panel:'#211d25',text:'#eeecf2',muted:'#b1a9bd',border:'#36313f',brand:'#dc96cd',soft:'#2c2230',support:'#9fbebc'}:{bg:'#faf9fd',panel:'#ffffff',text:'#292435',muted:'#70697c',border:'#e7e3ee',brand:'#8f1a7f',soft:'#f5edf6',support:'#497774'};
   const selected={...config,theme:'base',themeVariables:{fontFamily:'system-ui, sans-serif',fontSize:'15px',primaryColor:tone.panel,primaryTextColor:tone.text,primaryBorderColor:tone.border,lineColor:tone.muted,textColor:tone.text,clusterBkg:tone.soft,clusterBorder:tone.border,edgeLabelBackground:tone.bg},themeCSS:'.node rect,.node polygon,.cluster rect{filter:none!important}.cluster rect{rx:8;ry:8}.cluster-label text{font-weight:600}.edgePaths path,.flowchart-link{fill:none!important;stroke-width:1.2px!important}'};
   const classes=`\nclassDef first fill:${tone.panel},stroke:${tone.brand},color:${tone.text},stroke-width:1.3px,rx:6,ry:6;\nclassDef serial fill:${tone.panel},stroke:${tone.border},color:${tone.text},rx:6,ry:6;\nclassDef parallel fill:${tone.panel},stroke:${tone.support},color:${tone.text},rx:6,ry:6;\nclassDef core fill:${tone.bg},stroke:${tone.border},color:${tone.muted},stroke-dasharray:3 4,rx:6,ry:6;\nclassDef gate fill:${tone.bg},stroke:${tone.border},color:${tone.text};\nclassDef reuse fill:${tone.soft},stroke:${tone.border},color:${tone.text},rx:6,ry:6;\n`;

   const destination=path.join(folder,name.replace('.mmd',`-${mode}.svg`));
   let svg=await page.evaluate(async ({source,selected})=>{document.body.innerHTML='';mermaid.initialize({...selected,startOnLoad:false});return (await mermaid.render('farm-doc-diagram',source)).svg;},{source:source.toString('utf8')+classes,selected});
   if(/<\s*(?:script|foreignObject)\b|\son[a-z]+\s*=|<!DOCTYPE|<!ENTITY/i.test(svg))throw Error('Unsafe SVG structure: '+svg.match(/<\s*(?:script|foreignObject)\b|\son[a-z]+\s*=|<!DOCTYPE|<!ENTITY/i)?.[0]);
   if([...svg.matchAll(/(?:href|xlink:href)="([^"]*)"/g)].some(m=>!m[1].startsWith('#')))throw Error('External SVG reference');
   const view=svg.match(/viewBox="([^"]+)"/);if(!view)throw Error('Missing SVG viewBox');const [, ,width,height]=view[1].split(/\s+/).map(Number);
   svg=svg.replace(/<svg\b[^>]*>/,tag=>tag.replace(/\swidth="[^"]*"/,'').replace(/\sheight="[^"]*"/,'').replace(/>$/,` width="${Math.ceil(width)}" height="${Math.ceil(height)}">`));
   await fs.writeFile(destination,svg);entry.outputs[mode]={file:path.basename(destination),sha256:hash(svg),width:Math.ceil(width),height:Math.ceil(height)};
  }
  manifest.diagrams.push(entry);console.log(name);
 }
 await fs.writeFile(path.join(folder,'manifest.json'),JSON.stringify(manifest,null,2)+'\n');
}finally{await engine.close();}
