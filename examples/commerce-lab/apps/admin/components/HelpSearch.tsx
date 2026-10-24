import React from 'react';
export function HelpSearch({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  return <label>Search help<input aria-label="Search help" type="search" placeholder="Search titles and topics" value={value} onChange={event => onChange(event.target.value)} /></label>;
}
