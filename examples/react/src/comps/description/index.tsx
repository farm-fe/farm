import React, { Suspense } from 'react';

import imageUrl from '../../../assets/performance.png?inline';

const Clock = React.lazy(() => import('../clock'));

export function Description() {
  return <div className='description'>
    <p>Farm is a supper fast building engine written in rust.</p>
    <img src={imageUrl}></img>
    <Suspense fallback={'loading...'}> <Clock /></Suspense>
  </div>
}