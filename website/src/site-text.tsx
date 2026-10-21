import { useLocale } from '@farmfe/docs/client';
import React from 'react';
import messages from './locales/messages.json';

export type SiteMessageKey = keyof typeof messages.en;

/** Website presentation text; page metadata is resolved separately by Rust. */
export function SiteText({ id }: { id: SiteMessageKey }) {
  const locale = useLocale();
  const catalog = locale === 'zh' ? messages.zh : messages.en;
  return <>{catalog[id]}</>;
}
