import React, { useState } from 'react';
import { extensionCatalog } from '@commerce-extension';
import { Panel } from '../../../packages/ui';

export function ExtensionCatalog() {
  const [id, setId] = useState(extensionCatalog[0].id);
  const selected = extensionCatalog.find(category => category.id === id)!;
  const Form = selected.Form;
  return <Panel title="Generated category extensions">
    <p>{extensionCatalog.length} interactive categories · generated workload, separate from manual business components.</p>
    <label>Extension category<select aria-label="Extension category" value={id} onChange={event => setId(event.target.value)}>
      {extensionCatalog.map(category => <option key={category.id} value={category.id}>{category.title}</option>)}
    </select></label>
    <Form key={id} />
  </Panel>;
}
