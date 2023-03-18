import React, { Suspense } from 'react';
import './index.css';
import HomepageFeatures from '../features';
import { Button } from '../button';

// const Clock = React.lazy(() => import('../clock'));

export function Description() {
  return (
    <div className="description">
      <p>Super fast web building tool written in Rust.</p>

      <div
        style={{ display: 'flex', justifyContent: 'center', marginTop: '40px' }}
      >
        <Button to="https://farm-fe.github.io/docs/quick-start">
          Quick Start ⏱️
        </Button>
        <Button to="https://farm-fe.github.io/docs/why-farm">Why Farm?</Button>
      </div>
      <HomepageFeatures />

      {/* <Suspense fallback={'loading...'}>
        <Clock />
      </Suspense> */}
    </div>
  );
}
