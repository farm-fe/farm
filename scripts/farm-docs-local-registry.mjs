import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import http from 'node:http';
import {createHash} from 'node:crypto';

const hash=(algorithm,bytes,encoding='hex')=>createHash(algorithm).update(bytes).digest(encoding);

// The caller binds each manifest to its validated tarball; no extraction occurs here.
export async function startDocsLocalRegistry(packs) {
  assert(Array.isArray(packs)&&packs.length>0,'At least one validated package is required');
  const byName=new Map(),byRoute=new Map(),receipts=[],requests=[];
  for(const input of packs) {
    const manifest=JSON.parse(JSON.stringify(input.manifest));
    assert(/^@farmfe\/[a-z0-9][a-z0-9._-]*$/.test(manifest.name),'Registry accepts explicit Farm package names only');
    assert(/^\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?(?:\+[a-zA-Z0-9.-]+)?$/.test(manifest.version),'Exact package version is required');
    assert(!byName.has(manifest.name),'Duplicate package in registry: '+manifest.name);
    for(const field of ['dependencies','devDependencies','optionalDependencies','peerDependencies'])for(const range of Object.values(manifest[field]??{}))assert(typeof range==='string'&&!/^(?:workspace|file|link|portal):/i.test(range),'Unresolved workspace/local dependency in registry manifest');
    assert(typeof input.sha256==='string'&&/^[a-f0-9]{64}$/i.test(input.sha256),'Expected archive digest is required');
    const archive=await fs.realpath(input.archive),bytes=await fs.readFile(archive),sha256=hash('sha256',bytes);
    assert.equal(sha256,input.sha256.toLowerCase(),'Archive digest differs from pack receipt');
    const route='/tarballs/'+receipts.length+'.tgz';
    const item={manifest,archive,sha256,route,integrity:'sha512-'+hash('sha512',bytes,'base64'),shasum:hash('sha1',bytes)};
    byName.set(manifest.name,item);byRoute.set(route,item);
    receipts.push(Object.freeze({name:manifest.name,version:manifest.version,archive,sha256,integrity:item.integrity,bytes:bytes.length}));
  }
  let origin;
  async function respond(req,res) {
    const row={method:req.method,path:'',status:null};requests.push(row);
    function send(status,body,type='application/json') {
      row.status=status;
      const bytes=Buffer.isBuffer(body)?body:Buffer.from(JSON.stringify(body));
      res.writeHead(status,{'Content-Type':type,'Content-Length':bytes.length,'Cache-Control':'no-store'});
      res.end(req.method==='HEAD'?undefined:bytes);
    }
    if(!['GET','HEAD'].includes(req.method)){send(405,{error:'READ_ONLY_REGISTRY'});return;}
    let pathname;
    try {pathname=new URL(req.url,origin).pathname;row.path=pathname;}catch{send(404,{error:'NOT_FOUND'});return;}
    const archive=byRoute.get(pathname);
    if(archive) {
      try {
        const bytes=await fs.readFile(archive.archive);
        if(hash('sha256',bytes)!==archive.sha256){send(409,{error:'ARCHIVE_CHANGED'});return;}
        send(200,bytes,'application/octet-stream');
      }catch{send(409,{error:'ARCHIVE_UNAVAILABLE'});}
      return;
    }
    let name;
    try{name=decodeURIComponent(pathname.slice(1));}catch{send(404,{error:'NOT_FOUND'});return;}
    const item=byName.get(name)??[...byName.values()].find(pack=>name===pack.manifest.name+'/'+pack.manifest.version);
    if(!item){send(404,{error:'NOT_FOUND'});return;}
    const version={...item.manifest,dist:{tarball:origin+item.route,integrity:item.integrity,shasum:item.shasum}};
    send(200,name===item.manifest.name?{name:item.manifest.name,'dist-tags':{latest:item.manifest.version},versions:{[item.manifest.version]:version}}:version);
  }
  const server=http.createServer((req,res)=>{
    void respond(req,res).catch(()=>{if(!res.headersSent)res.writeHead(500);res.end();});
  });
  server.on('clientError',(_error,socket)=>socket.destroy());
  await new Promise((resolve,reject)=>{
    server.once('error',reject);
    server.listen(0,'127.0.0.1',()=>{server.removeListener('error',reject);resolve();});
  });
  origin='http://127.0.0.1:'+server.address().port;
  let closing;
  return {
    url:origin,
    get packs(){return [...receipts];},
    get requests(){return requests.map(row=>({...row}));},
    close(){
      if(!closing)closing=new Promise((resolve,reject)=>{
        server.close(error=>error?reject(error):resolve());
        server.closeAllConnections();
      });
      return closing;
    }
  };
}
