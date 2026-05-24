import React from 'react';

/**
 * Demonstrates a CSS class defined via `@apply` in index.css.
 * Provides a stable selector for the e2e spec to assert against the
 * generated stylesheet.
 */
const AppliedAlert: React.FC<{ message: string }> = ({ message }) => {
  return (
    <div data-testid="applied-alert" className="applied-alert">
      <strong>Notice:</strong> {message}
    </div>
  );
};

export default AppliedAlert;
