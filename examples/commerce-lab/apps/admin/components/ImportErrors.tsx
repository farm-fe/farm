import React from 'react';

export function ImportErrors({ errors }: {
  errors: Array<{ row: number; field: string; message: string }>;
}) {
  if (!errors.length) return null;
  return <ul data-testid="import-errors">{errors.map((error, index) =>
    <li key={index}>Row {error.row}: {error.field} — {error.message}</li>)}</ul>;
}
