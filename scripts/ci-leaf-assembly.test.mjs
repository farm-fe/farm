import test from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {bindCiAssemblyOrigins} from './assemble-v2-native-artifacts.mjs';
const digest=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
function fixture(){
 const current='c'.repeat(40),old='a'.repeat(40),sha256='b'.repeat(64);
 const records=[{name:'@farmfe/core-linux-x64-gnu',platform:'linux-x64-gnu',sha256,artifact:'logical-staging-route'},
 {name:'@farmfe/core-darwin-arm64',platform:'darwin-arm64',sha256,artifact:'actual-current-native'}];
 const binding={kind:'ci-consumer-materialization-not-a-new-build',chain:'ci',profile:'ci',abi:'linux-x64-gnu',consumerSHA:current,originBuildSHA:old,
 originRunId:2,originArtifactId:3,originArtifactDigest:'sha256:'+'d'.repeat(64),acceptanceInherited:false,records:[{name:records[0].name,sha256}]};
 return {current,old,records,binding};
}
test('assembly explicitly preserves original producer and current consumer without a new-build claim',()=>{
 const value=fixture(),body=Buffer.from(JSON.stringify(value.binding));
 const rows=bindCiAssemblyOrigins(value.records,body,digest(body),value.current);
 assert.equal(rows[0].producer.originBuildSHA,value.old);assert.equal(rows[0].producer.consumerSHA,value.current);
 assert.equal(rows[0].artifact,'ci-consumer-materialization:3');assert.equal(rows[1].artifact,'actual-current-native');
 assert.equal(rows[0].producer.acceptanceInherited,false);
});
test('altered commitment, partial inventory, wrong bytes and cross-chain provenance are rejected',()=>{
 for(const mutate of [v=>{v.binding.chain='release';},v=>{v.binding.profile='release';},v=>{v.binding.records=[];},v=>{v.binding.records[0].sha256='0'.repeat(64);},v=>{v.binding.consumerSHA=v.old;}]){
  const value=fixture();mutate(value);const body=Buffer.from(JSON.stringify(value.binding));
  assert.throws(()=>bindCiAssemblyOrigins(value.records,body,digest(body),value.current));
 }
 const value=fixture(),body=Buffer.from(JSON.stringify(value.binding));assert.throws(()=>bindCiAssemblyOrigins(value.records,body,'0'.repeat(64),value.current));
});
