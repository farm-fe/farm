import React from 'react';
import { useLocale, useSite } from '@farmfe/docs/client';
import { SearchPage } from '@farmfe/docs-theme-default';
import type { PageMetadata } from '@farmfe/docs';

export const metadata = {
  title: 'Search the documentation',
  titleByLocale: { zh: '在文档中搜索' }
} satisfies PageMetadata;

export default function Search() {
  const locale = useLocale();
  const labels = locale === 'zh' ? {
    input: '搜索文档', submit: '搜索', idle: '输入关键字搜索。',
    loading: '正在搜索…', error: '搜索失败，请重试。', empty: '没有搜索结果。',
    retry: '重新搜索', results: '搜索结果', unconfigured: '尚未配置搜索。',
    count: (count: number) => `${count} 条结果。`
  } : undefined;
  return <>
    <h1>{locale === 'zh' ? '在文档中搜索' : 'Search the documentation'}</h1>
    <SearchPage site={useSite()} labels={labels} />
  </>;
}
