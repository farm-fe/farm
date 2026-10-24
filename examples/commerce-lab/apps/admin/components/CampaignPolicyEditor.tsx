import React, { useState } from 'react';
export function CampaignPolicyEditor({ maxDiscount, onSave }: { maxDiscount: number; onSave: (limit: number) => void }) {
  const [draft, setDraft] = useState(String(maxDiscount)); const [error, setError] = useState(''); const [saved, setSaved] = useState(false);
  return <form className="panel" onSubmit={event => { event.preventDefault(); try {
    if (!/^\d+$/.test(draft)) throw new Error('Maximum discount must be a whole number from 1 to 80.');
    onSave(Number(draft)); setError(''); setSaved(true);
  } catch (failure) { setError(failure instanceof Error ? failure.message : String(failure)); setSaved(false); } }}>
    <h2>Edit merchant discount limit</h2><label>Maximum campaign discount<input aria-label="Maximum campaign discount" inputMode="numeric" value={draft} onChange={event => { setDraft(event.target.value); setSaved(false); }} /></label>
    {error && <p role="alert">{error}</p>}{saved && <p role="status">Campaign policy saved.</p>}<button type="submit">Save campaign policy</button>
  </form>;
}
