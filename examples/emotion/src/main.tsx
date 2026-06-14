import { useState } from 'react';
import { EmotionCard } from './emotion';
import { StyledComponentsCard } from './styled-components';
import { StyledJsxCard } from './styled-jsx';

const obj = {
  a: 1
};

const color = 'white';

export function Main() {
  const [count, setCount] = useState(0);

  // obj.a //OK
  const a = obj?.a;

  return (
    <>
      <EmotionCard onClick={() => setCount((c) => c + 1)}>{a}: {count}</EmotionCard>
      <StyledComponentsCard />
      <StyledJsxCard />
    </>
  );
}
