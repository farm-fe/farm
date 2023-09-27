import { useState } from 'react';
import { css } from '@emotion/react';

const obj = {
  a: 1
};

const color = 'white';

export function Main() {
  const [count, setCount] = useState(0);

  // obj.a //OK
  const a = obj?.a;

  return (
    <div
      onClick={() => setCount((c) => c + 1)}
      css={css`
        padding: 32px;
        background-color: hotpink;
        font-size: 24px;
        border-radius: 4px;
        &:hover {
          color: ${color};
        }
      `}
    >
      {a}: {count}
    </div>
  );
}
