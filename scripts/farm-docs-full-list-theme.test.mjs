import assert from 'node:assert/strict';
import {test} from 'node:test';
import {createRequire} from 'node:module';
import {BlogPage} from '../packages/docs-theme-default/dist/index.js';
const require=createRequire(new URL('../packages/docs/package.json',import.meta.url));
const React=require('react');
const {renderToStaticMarkup}=require('react-dom/server');
const element=React.createElement;
function props(fullContent=true){return {
 site:{},page:{key:'listing',title:'Blog',collection:'news',blogView:{kind:'blog-index',posts:[],fullContent}},
 data:{pages:[],sidebars:[],blogIndexes:[],home:'/'},components:{},
 children:element(React.Fragment,null,element('h1',null,'Blog'),element('p',null,'No posts.')),
};}
test('full listing renders provided original bodies once and does not repeat the compact list',()=>{
 const input=props();input.listingPosts=[{page:{key:'article',title:'Release',routePath:'/blog/release/'},children:element('p',{'data-testid':'original-body'},'2024')}];
 const html=renderToStaticMarkup(element(BlogPage,input));
 assert.equal((html.match(/data-testid="original-body"/g)||[]).length,1);
 assert.match(html,/>2024</);assert.match(html,/href="\/blog\/release\/"/);
 assert.match(html,/href="\/blog\/release\/">Release<\/a>/,'article link preserves its title without an English-only prefix');
 assert.doesNotMatch(html,/No posts/);
});
test('empty full listing retains its generated empty state with one heading',()=>{
 const input=props();input.listingPosts=[];
 const html=renderToStaticMarkup(element(BlogPage,input));
 assert.equal((html.match(/<h1>/g)||[]).length,1);assert.match(html,/No posts/);
});
test('full listing refuses missing article payload while link mode keeps generated content',()=>{
 assert.throws(()=>renderToStaticMarkup(element(BlogPage,props())),/DOCS_BLOG_CONTENT/);
 const html=renderToStaticMarkup(element(BlogPage,props(false)));
 assert.match(html,/No posts/);assert.equal((html.match(/<h1>/g)||[]).length,1);
});
