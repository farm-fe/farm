import React, { Suspense } from './dep';

const LazyComp: any = React.lazy(() => Promise.resolve({ default: () => <div>Lazy</div> }));

export function Description() {
  console.trace('In Description, the sourcemap should be correct');

  return <Suspense fallback={<div>Loading...</div>}><LazyComp /></Suspense>;
}
