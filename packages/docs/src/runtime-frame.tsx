import type { ReactNode } from 'react';

/** Keep the same sibling slots during SSR and hydration for React useId. */
export function RuntimeFrame({ content, failure = null, pending = null }: { content: ReactNode; failure?: ReactNode; pending?: ReactNode }) {
  return <>{content}{failure}{pending}</>;
}
