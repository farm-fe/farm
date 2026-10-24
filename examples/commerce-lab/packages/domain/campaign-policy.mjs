function requireMerchant(state, tenant) {
  if (!state.tenants.some(row => row.id === tenant)) throw new Error('Unknown merchant.');
}
function validLimit(value) {
  if (!Number.isInteger(value) || value < 1 || value > 80) throw new Error('Maximum discount must be a whole number from 1 to 80.');
}
export function campaignPolicy(state, tenant) {
  requireMerchant(state, tenant);
  const maxDiscount = state.campaignPolicies?.[tenant]?.maxDiscount ?? 80;
  validLimit(maxDiscount);
  return { maxDiscount };
}
export function updateCampaignPolicy(state, tenant, maxDiscount) {
  requireMerchant(state, tenant); validLimit(maxDiscount);
  return { ...state, campaignPolicies: { ...state.campaignPolicies, [tenant]: { maxDiscount } } };
}
