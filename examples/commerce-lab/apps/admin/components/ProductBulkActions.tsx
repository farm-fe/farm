import React from 'react';
import { Button as BulkButton, Tooltip as SelectionTooltip } from '@arco-design/web-react';
export function ProductBulkActions({ selected, role, onPublish, onClear }: { selected: number; role: string; onPublish: () => void; onClear: () => void }) {
  return <div className="toolbar"><a className="button" href="#/import">Import collection</a>{role === 'admin' && <><SelectionTooltip content="Selection includes products hidden by filters."><BulkButton type="primary" disabled={!selected} onClick={onPublish}>Publish selected ({selected})</BulkButton></SelectionTooltip><BulkButton disabled={!selected} onClick={onClear}>Clear product selection</BulkButton><span>Selection includes products hidden by filters.</span></>}</div>;
}
