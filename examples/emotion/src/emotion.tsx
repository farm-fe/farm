import { css } from '@emotion/react';
import type { PropsWithChildren } from 'react';

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

export function EmotionCard({
  children,
  onClick,
}: PropsWithChildren<{ onClick: () => void }>) {
  return (
    <div data-testid="emotion-card" onClick={onClick} css={emotionCard}>
      {children}
    </div>
  );
}
