import assert from 'node:assert/strict';

function selector(values,actual) {
  if(values===undefined||values.length===0)return true;
  assert(Array.isArray(values)&&values.every(value=>typeof value==='string'),'Invalid platform selector metadata');
  if(values.includes('!'+actual))return false;
  const positive=values.filter(value=>!value.startsWith('!'));
  return positive.length===0||positive.includes('any')||positive.includes(actual);
}
function compatible(pkg,target) {
  if(pkg.kind==='platform')assert(Array.isArray(pkg.os)&&pkg.os.length&&Array.isArray(pkg.cpu)&&pkg.cpu.length,'Platform package metadata needs os/cpu: '+pkg.name);
  return selector(pkg.os,target.platform)&&selector(pkg.cpu,target.arch)&&(target.platform!=='linux'||selector(pkg.libc,target.libc));
}

// Declaration planning only. No binary probing, semver solver or installation.
export function planDocsTargetClosure(inventory,target,{roots=inventory.roots}={}) {
  assert(typeof target?.platform==='string'&&typeof target.arch==='string','Explicit target platform/arch is required');
  if(target.platform==='linux')assert(['glibc','musl'].includes(target.libc),'Explicit Linux libc glibc/musl is required');
  assert(Array.isArray(roots)&&roots.length,'Explicit nonempty roots are required');
  const local=new Map(inventory.local.map(pkg=>[pkg.name,pkg]));
  assert.equal(local.size,inventory.local.length,'Duplicate inventory package name');
  const edges=new Map();
  for(const edge of inventory.edges){if(!edges.has(edge.from))edges.set(edge.from,[]);edges.get(edge.from).push(edge);}
  const queue=[...roots],visited=new Set(),required=[],skipped=new Map(),external=[];
  while(queue.length) {
    const name=queue.shift();if(visited.has(name))continue;visited.add(name);
    const pkg=local.get(name);assert(pkg,'Unresolved Farm package/root: '+name);
    assert(compatible(pkg,target),'No compatible platform for required package: '+name);
    required.push({...pkg});
    const dependencies=edges.get(name)??[];
    const native=dependencies.filter(edge=>local.get(edge.to)?.kind==='platform');
    if(native.length) {
      for(const edge of native){const child=local.get(edge.to);assert(edge.field==='optionalDependencies'&&edge.range===child.version&&child.version===pkg.version,'Native platform must use exact parent version: '+edge.to);}
      const matches=native.filter(edge=>compatible(local.get(edge.to),target));
      assert(matches.length>0,'No compatible native platform declared for '+name);
      assert.equal(matches.length,1,'Ambiguous native platform declarations for '+name);
    }
    for(const edge of dependencies) {
      assert(['dependencies','optionalDependencies','peerDependencies'].includes(edge.field),'Unexpected inventory dependency section');
      const child=local.get(edge.to);
      if(!child){assert(!edge.to.startsWith('@farmfe/'),'Unresolved Farm dependency: '+edge.to);external.push({...edge});continue;}
      if(!compatible(child,target)) {
        assert(edge.field==='optionalDependencies','No compatible platform for mandatory dependency: '+edge.to);
        skipped.set(child.name,{name:child.name,version:child.version,reason:'Declared optional package incompatible with target metadata'});
      }else queue.push(child.name);
    }
  }
  return {target:{...target},roots:[...roots],required:required.sort((a,b)=>a.name.localeCompare(b.name)),skipped:[...skipped.values()].sort((a,b)=>a.name.localeCompare(b.name)),external,
    scope:'Declared target closure only; not tarball availability, npm resolution, loader, binary ABI or platform acceptance.'};
}

// Call only with packs already validated through archive/directory provenance checks.
export function selectDocsRegistryPacks(plan,packs) {
  const required=new Map(plan.required.map(pkg=>[pkg.name,pkg])),available=new Map();
  for(const pack of packs) {
    const name=pack.manifest?.name;
    assert(!available.has(name),'Duplicate registry pack: '+name);
    assert(required.has(name),'Unplanned registry pack: '+name);
    assert.equal(pack.manifest.version,required.get(name).version,'Registry pack version differs: '+name);
    for(const field of ['os','cpu','libc'])assert.deepEqual(pack.manifest[field],required.get(name)[field],'Registry platform metadata differs: '+name+'/'+field);
    available.set(name,pack);
  }
  const missing=[...required.keys()].filter(name=>!available.has(name));
  assert.equal(missing.length,0,'Missing required registry packs: '+missing.join(', '));
  return plan.required.map(pkg=>available.get(pkg.name));
}
