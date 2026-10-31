import React from 'react';

export function ImportSource({ csv, onDownload, onUpload, onChange }: {
  csv: string;
  onDownload: () => void;
  onUpload: (file: File) => Promise<void>;
  onChange: (csv: string) => void;
}) {
  return <><button onClick={onDownload}>Download template</button>
    <label>Upload CSV<input type="file" accept=".csv" onChange={event => {
      const file = event.target.files?.[0];
      if (file) void onUpload(file);
    }} /></label>
    <label>CSV contents<textarea value={csv} onChange={event => onChange(event.target.value)} rows={10} /></label>
  </>;
}
