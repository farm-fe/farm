import dep1 from './dep1';

export const dep = 'dep';

export default function () {
  return dep1();
}

const should_be_removed = 'should be removed';

console.log('side effect in dep.ts');
