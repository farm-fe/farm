import React from 'react';
export function CampaignPolicySummary({ maxDiscount, channels }: { maxDiscount: number; channels: string[] }) {
  return <section aria-label="Campaign policy"><h2>Discount policy</h2><p>Maximum discount: {maxDiscount}%</p><p>New campaigns must meet the current merchant policy. Existing campaigns keep their published offer.</p><h2>Enabled channels</h2><p>{channels.join(', ')}</p></section>;
}
