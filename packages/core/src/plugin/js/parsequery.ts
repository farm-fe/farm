export interface VueQuery {
  vue?: boolean;
  src?: string;
  type?: 'script' | 'template' | 'style' | 'custom';
  index?: number;
  lang?: string;
  raw?: boolean;
  url?: boolean;
  scoped?: boolean;
  id?: string;
}

export function parseVueRequest(id: string): {
  filename: string;
  query: VueQuery;
} {
  const [filename, rawQuery] = id.split(`?`, 2);
  const query = Object.fromEntries(new URLSearchParams(rawQuery)) as VueQuery;
  const langMap = new Map([
    ['css', 'style'],
    ['scss', 'style'],
    ['less', 'style']
  ]);
  if (query.lang != null) {
    query.type = langMap.get(query.lang) as VueQuery['type'];
  }
  if (query.vue != null) {
    query.vue = true;
  }
  if (query.index != null) {
    query.index = Number(query.index);
  }
  if (query.raw != null) {
    query.raw = true;
  }
  if (query.url != null) {
    query.url = true;
  }
  if (query.scoped != null) {
    query.scoped = true;
  }
  return {
    filename,
    query
  };
}
