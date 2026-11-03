import assert from 'node:assert/strict';
const time=value=>{const n=Date.parse(value);assert.ok(Number.isFinite(n),'Invalid producer time');return n;};

export async function attemptJobs(api,prefix,runId,attempt){
  const rows=[];
  for(let page=1;page<=20;page++){
    const result=await api(`${prefix}/actions/runs/${runId}/attempts/${attempt}/jobs?per_page=100&page=${page}`);
    assert.ok(Array.isArray(result.jobs),'Invalid attempt jobs');rows.push(...result.jobs);if(result.jobs.length<100)return rows;
  }
  throw Error('Producer jobs exceed pagination bound');
}
export async function observeAttempts(api,prefix,run){
  assert.ok(Number.isSafeInteger(run.run_attempt)&&run.run_attempt>0&&run.run_attempt<=32,'Attempt exceeds verification bound');
  const attempts=[];
  for(let attempt=run.run_attempt;attempt>=1;attempt--){
    const execution=await api(`${prefix}/actions/runs/${run.id}/attempts/${attempt}`);
    for(const key of ['id','path','head_sha','event'])assert.equal(execution[key],run[key],'Original execution differs');
    assert.equal(execution.run_attempt,attempt);assert.equal(execution.repository.id,run.repository.id);
    if(run.head_repository)assert.equal(execution.head_repository?.id,run.head_repository.id);
    attempts.push({attempt,runId:run.id,jobs:await attemptJobs(api,prefix,run.id,attempt)});
  }
  return attempts;
}
export function actualJob(attempts,name){
  for(const execution of attempts){
    const matches=execution.jobs.filter(job=>job.name===name);assert.ok(matches.length<=1,'Ambiguous actual producer');
    const job=matches[0];if(!job)continue;
    assert.ok(Number.isSafeInteger(job.id)&&job.id>0&&job.run_id===execution.runId,'Producer job identity differs');
    assert.ok(job.started_at&&job.completed_at&&job.status==='completed'&&job.conclusion==='success','Latest actual measurement producer did not succeed');
    const previous=attempts.find(row=>row.attempt===execution.attempt-1);
    const prior=previous?.jobs.filter(row=>row.name===name)??[];assert.ok(prior.length<=1,'Ambiguous previous producer');
    if(prior[0]?.started_at===job.started_at&&prior[0]?.completed_at===job.completed_at){
      assert.ok(prior[0].id===job.id&&prior[0].status==='completed'&&prior[0].conclusion==='success','Unproven carried measurement producer');continue;
    }
    if(prior[0]?.completed_at)assert.ok(time(job.started_at)>=time(prior[0].completed_at),'Producer execution times overlap across attempts');
    return {...execution,job};
  }
  throw Error('Actual measurement producer absent');
}
