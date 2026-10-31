import React from 'react';

export function ConnectionControls({ mode, resource, onMode, onResource }: {
  mode: string; resource: string;
  onMode: (value: string) => void; onResource: (value: string) => void;
}) {
  return <div className="toolbar"><label>API resource<select aria-label="API resource" value={resource} onChange={event => onResource(event.target.value)}>
    <option value="products">Merchant products</option><option value="orders">Merchant orders</option>
    <option value="metrics">Business metrics</option><option value="campaigns">Published campaigns</option>
  </select></label>
    <label>Response mode<select aria-label="Response mode" value={mode} onChange={event => onMode(event.target.value)}>
      <option value="success">Healthy connection</option><option value="timeout">Request timeout</option>
      <option value="forbidden">Permission denied</option></select></label></div>;
}
