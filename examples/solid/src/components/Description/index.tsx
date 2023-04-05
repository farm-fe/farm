import { type Component, Suspense, lazy } from 'solid-js';
import { HomepageFeatures } from '../Features';
import { Button } from '../Button';

import './index.css';

const Clock = lazy(() => import('../Clock'));

export const Description: Component = () => {
  console.trace('In Description, the sourcemap should be correct');

  return (
    <div class="description">
      <p>Super fast web building tool written in Rust.</p>

      <div
        style={{
          display: 'flex',
          'justify-content': 'center',
          'margin-top': '40px',
        }}
      >
        <Button to="https://farm-fe.github.io/docs/quick-start">
          Quick Start ⏱️
        </Button>
        <Button to="https://farm-fe.github.io/docs/why-farm">Why Farm?</Button>
      </div>
      <HomepageFeatures />
      <Suspense fallback={'loading...'}>
        <Clock />
      </Suspense>
    </div>
  );
};
