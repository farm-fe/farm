import React, { Suspense } from 'react';

const Clock = React.lazy(() => import('../clock'));

export function Description() {
  return <div className='description'>
    <p>Farm is a supper fast building engine written in rust.</p>

    <Suspense fallback={'loading...'}> <Clock /></Suspense>
  </div>
}