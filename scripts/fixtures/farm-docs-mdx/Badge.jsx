import React, { useState } from 'react';
export default function Badge({label, children}) {
  const [count, setCount] = useState(0);
  return <section data-badge={label}>{children}<button onClick={() => setCount(count + 1)}>{`Count ${count}`}</button></section>;
}
