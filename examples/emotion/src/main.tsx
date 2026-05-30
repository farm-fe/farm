import { useState } from 'react';
import { css } from '@emotion/react';

const obj = {
  a: 1
};

const color = 'white';

const emotionCard = css`
  padding: 32px;
  background-color: hotpink;
  font-size: 24px;
  border-radius: 4px;
  &:hover {
    color: ${color};
  }
`;

export function Main() {
  const [count, setCount] = useState(0);

  // obj.a //OK
  const a = obj?.a;

  return (
    <div
      data-testid="emotion-card"
      onClick={() => setCount((c) => c + 1)}
      css={emotionCard}
    >
      {a}: {count}
    </div>
  );
}
