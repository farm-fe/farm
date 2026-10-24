import React from 'react';

export function ImportTypesPreview({ source }: { source: string }) {
  const download = () => {
    const url = URL.createObjectURL(new Blob([source], { type: 'text/typescript' }));
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'product-import-types.ts';
    anchor.click();
    URL.revokeObjectURL(url);
  };
  return <>
    <pre data-testid="import-types-code"><code>{source}</code></pre>
    <button type="button" onClick={download}>Download TypeScript types</button>
  </>;
}
