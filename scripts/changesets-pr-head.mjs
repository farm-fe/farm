import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';

const baseRef = 'refs/heads/main';
const headRef = 'refs/heads/changeset-release/main';
const sha = /^[a-f0-9]{40}$/;

export async function waitForVersionPrHead({
  repository, number, base, previous, expected, readPR, readRefs,
  wait = () => new Promise(resolve => setTimeout(resolve, 2000)), maxAttempts = 5,
}) {
  assert.match(repository, /^[\w.-]+\/[\w.-]+$/);
  assert.ok(Number.isSafeInteger(number) && number > 0);
  for (const value of [base, previous, expected]) assert.match(value, sha);
  assert.notEqual(previous, expected);
  assert.ok(Number.isInteger(maxAttempts) && maxAttempts >= 1 && maxAttempts <= 5);
  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    const pr = await readPR();
    assert.ok(pr.number === number && pr.state === 'open' && /^chore: release packages/.test(pr.title)
      && pr.base?.repo?.full_name === repository && pr.head?.repo?.full_name === repository
      && `refs/heads/${pr.base.ref}` === baseRef && `refs/heads/${pr.head.ref}` === headRef,
    'Version PR identity changed');
    assert.equal(pr.base.sha, base, 'Version PR base moved concurrently');
    assert.ok(pr.head.sha === expected || pr.head.sha === previous, 'Version PR head moved concurrently');
    // Read Git after REST, so an old PR projection cannot hide an observable
    // branch change or cause us to accept a different seal.
    const refs = await readRefs();
    assert.equal(refs.base, base, 'Version PR remote main moved concurrently');
    assert.equal(refs.head, expected, 'Version PR remote head moved concurrently');
    if (pr.head.sha === expected) return pr;
    if (attempt + 1 < maxAttempts) await wait();
  }
  throw new Error('Version PR head visibility did not converge to the expected seal');
}

export function confirmVersionPrHead(root, identity) {
  const run = (command, args) => execFileSync(command, args, { cwd: root, encoding: 'utf8', timeout: 10000 }).trim();
  return waitForVersionPrHead({
    ...identity,
    readPR: () => JSON.parse(run('gh', ['api', `repos/${identity.repository}/pulls/${identity.number}`])),
    readRefs: () => {
      const rows = run('git', ['ls-remote', '--exit-code', '--refs', 'origin', baseRef, headRef]).split('\n');
      const refs = new Map(rows.map(row => { const [oid, ref] = row.split('\t'); return [ref, oid]; }));
      assert.equal(rows.length, 2, 'Missing or ambiguous version PR remote refs');
      return { base: refs.get(baseRef), head: refs.get(headRef) };
    },
  });
}
