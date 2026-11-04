"""Exact registry version selection; immutable source and frozen observation inputs."""
# cspell:words importlib urllib npmjs tomllib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import urllib.error
import urllib.parse


def load(name, file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

chain=load('registry_selection_chain','incremental-chain-plan.py')
registry_module=load('registry_selection_reader','incremental-registry-evidence.py')


def check(value,message):
    if not value:raise ValueError(message)


def records(source,kind):
    nodes,issues=chain.repository_graph(source['files'])
    check(not issues,'Registry selection requires complete dependency graph')
    family=chain.independent_family(source['files'],nodes)
    actual={key:row for key,row in family.items() if row['kind']==kind}
    if kind=='npm':
        # npm payload assembly retains its complete reviewed candidate inventory.
        # Cargo versions are independently declared by the fixed SDK source graph;
        # release-plz updates must not depend on stale npm candidate metadata.
        declared=chain.family_records(json.loads(source['files']['docs/releases/v2-candidate.json']))
        fields=['kind','name','version','manifestPath','owner','platform']
        expected={key:row for key,row in declared.items() if row['kind']==kind}
        check(set(actual)==set(expected) and all(all(row.get(field)==expected[key].get(field) for field in fields) for key,row in actual.items()),'Declared full version family differs from source')
    chain.validate_family_sources(actual,source['files'])
    return actual


class Registry:
    def __init__(self, reader=None):
        self.reader=reader or registry_module.RegistryReadOnly()

    def lookup(self,row):
        kind,name,version=row['kind'],row['name'],row['version']
        check(kind in ['npm','cargo'],'Unknown publication registry')
        check(re.fullmatch(r'\d+\.\d+\.\d+',version),'Exact stable version required')
        check(re.fullmatch(r'(?:@[a-z0-9._-]+/)?[a-zA-Z0-9._-]+',name),'Unsafe registry package name')
        url=('https://registry.npmjs.org/'+urllib.parse.quote(name,safe='')+'/'+version if kind=='npm' else 'https://crates.io/api/v1/crates/'+name+'/'+version)
        try:data=self.reader.json(url)
        except urllib.error.HTTPError as error:
            if error.code==404:return {'status':'missing','name':name,'version':version}
            raise
        if kind=='npm':
            check(data.get('name')==name and data.get('version')==version,'Registry metadata identity differs')
            dist=data['dist'];archive=dist['tarball']
            check(urllib.parse.urlsplit(archive).hostname=='registry.npmjs.org','Registry archive origin differs')
            actual=self.reader.archive(archive)
            integrity=dist.get('integrity','').split()
            check(('sha512-'+actual['sha512'] in integrity) if integrity else dist.get('shasum')==actual['sha1'],'Registry archive integrity differs')
            metadata={key:data.get(key,{}) for key in ['dependencies','optionalDependencies','peerDependencies']}
        else:
            item=data['version'];check(item.get('crate')==name and item.get('num')==version and item.get('yanked') is False,'Cargo version identity or yanked state differs')
            actual=self.reader.archive('https://static.crates.io/crates/'+name+'/'+name+'-'+version+'.crate')
            check(item['checksum']==actual['sha256'],'Cargo archive checksum differs');metadata={}
        return {'status':'published','name':name,'version':version,'archiveSha256':actual['sha256'],'metadata':metadata,**({'integrity':'sha512-'+actual['sha512']} if kind=='npm' else {})}


def collect(source,kind,lookup):
    family=records(source,kind)
    rows={key:lookup(row) for key,row in sorted(family.items())}
    return {'schemaVersion':1,'kind':'exact-registry-version-observation','sourceSHA':source['sha'],'sourceTree':source['tree'],'registryKind':kind,'packages':rows}


def validate(source,kind,snapshot):
    family=records(source,kind)
    if snapshot.get('stateBaseline') is not None:chain.exact(snapshot['stateBaseline'],64)
    check(snapshot.get('schemaVersion')==1 and snapshot.get('kind')=='exact-registry-version-observation' and snapshot.get('registryKind')==kind,'Registry observation policy differs')
    check(snapshot.get('sourceSHA')==source['sha'] and snapshot.get('sourceTree')==source['tree'],'Registry observation source differs')
    check(set(snapshot['packages'])==set(family),'Registry observation full family differs')
    for key,row in family.items():
        observed=snapshot['packages'][key]
        check(observed.get('name')==row['name'] and observed.get('version')==row['version'],'Registry observation version differs')
        check(observed.get('status') in ['missing','published'],'Registry error is not absence')
        if observed['status']=='published':
            check(re.fullmatch('[a-f0-9]{64}',observed.get('archiveSha256','')) and isinstance(observed.get('metadata'),dict),'Published version evidence incomplete')
    return family


def verify_retained(source,snapshot,lookup):
    family=validate(source,snapshot['registryKind'],snapshot)
    for key,row in family.items():
        previous=snapshot['packages'][key]
        if previous['status']=='published':
            check(lookup(row)==previous,'Retained published version changed, disappeared or differs')


def npm_dependencies(source,snapshot):
    """Check internal runtime/optional/peer declarations against the complete family.

    External dependencies remain subject to frozen-lock and candidate install checks.
    Published packages are checked using their registry declarations, not new source.
    """
    family=validate(source,'npm',snapshot);by_name={row['name']:row for row in family.values()};checks=[]
    for key,row in family.items():
        observed=snapshot['packages'][key]
        manifest=observed['metadata'] if observed['status']=='published' else json.loads(source['files'][row['manifestPath']])
        for section in ['dependencies','optionalDependencies','peerDependencies']:
            for name,value in manifest.get(section,{}).items():
                if name not in by_name:continue
                check(isinstance(value,str),'Invalid internal dependency range')
                if value.startswith('workspace:'):
                    value=value[len('workspace:'):]
                    if value in ['*','^','~']:value=by_name[name]['version']
                checks.append([by_name[name]['version'],value,row['name'],name])
    for version,requirement,package,dependency in checks:
        check(stable_satisfies(version,requirement),'Internal publication dependency incompatible: '+package+' -> '+dependency)


def stable_satisfies(version,requirement):
    """Deliberately bounded stable family ranges; unsupported syntax fails closed.

    Family manifests currently use exact versions and workspace:* / workspace:^.
    Published normalized equivalents may use * / ^ / ~. No tag or prerelease can
    silently authorize a different family member.
    """
    check(re.fullmatch(r'\d+\.\d+\.\d+',version),'Stable family dependency required')
    if requirement=='*':return True
    match=re.fullmatch(r'([~^=]?)(\d+)\.(\d+)\.(\d+)',requirement)
    check(match is not None,'Unsupported internal publication range: '+requirement)
    prefix=match[1];floor=tuple(map(int,match.groups()[1:]));actual=tuple(map(int,version.split('.')))
    if prefix in ['', '=']:return actual==floor
    if prefix=='~':ceiling=(floor[0],floor[1]+1,0)
    elif floor[0]:ceiling=(floor[0]+1,0,0)
    elif floor[1]:ceiling=(0,floor[1]+1,0)
    else:ceiling=(0,0,floor[2]+1)
    return floor<=actual<ceiling



def plan(source,family,snapshot):
    declared=validate(source,'npm',snapshot);npm_dependencies(source,snapshot)
    # Reuse the common complete-family and graph validation and dependency walk.
    result=chain.plan_chain('release',{'sha':source['sha'],'tree':source['tree']},source['files'],family,[],source_modes=source['modes'],publication_scope='npm')
    selected=sorted(key for key,row in snapshot['packages'].items() if row['status']=='missing')
    result.update(selectionPolicy='registry-exact-version-v1',registrySnapshot=snapshot,baselineSHA=None,
                  build=selected,publish=selected,versionRequired=[],unchanged=sorted(set(declared)-set(selected)),
                  publicationBlocked=False,publicationBlockReason=None,registryCollisionChecks=selected,
                  reasons=['exact-registry-version-difference'],fullBuild=len(selected)==len(declared),
                  impactTrace=[],directNodes=[],affectedNodes=[],testNodes=[],sameVersionRebuildPolicy='build-dependencies-without-republishing')
    result['baselineBinding']=snapshot.get('stateBaseline')
    result['registryRetained']={key:{**declared[key],**row} for key,row in snapshot['packages'].items() if row['status']=='published'}
    nodes,issues=chain.repository_graph(source['files']);check(not issues,'Publication graph unresolved')
    chain.build_closure(result,declared,nodes)
    result['planHash']=chain.digest({key:value for key,value in result.items() if key!='planHash'})
    return result
