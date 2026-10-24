import React from 'react';

export function ImportValidationActions({ busy, onValidate, onCancel }: {
  busy: boolean;
  onValidate: () => void;
  onCancel: () => void;
}) {
  return <><button className="primary" disabled={busy} onClick={onValidate}>
    {busy ? 'Validating…' : 'Validate in Worker'}</button>
    <button disabled={!busy} onClick={onCancel}>Cancel validation</button></>;
}
