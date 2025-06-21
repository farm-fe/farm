import React, { Suspense, lazy } from 'react';
import './main.css';

console.log(process.env.FARM_BASE_TEST);

const LazyWelcome = lazy(() => import('./components/welcome'));

export function Main() {
  return (
    <>
      <Suspense fallback={<div>Loading...</div>}>
        <LazyWelcome />
      </Suspense>
    </>
  );
}
