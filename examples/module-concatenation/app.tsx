import React from './react';
import util1 from './util1.cjs';

import commonInner from './common.mjs';

function common() {
  return 'common-outer';
}

export function App() {
  return (
    <div>
      <h1>{util1.util1() + util1.util2()}</h1>
      <h1>{commonInner()}</h1>
    </div>
  );
}

