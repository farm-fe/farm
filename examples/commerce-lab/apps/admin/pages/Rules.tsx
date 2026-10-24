import React from 'react';
import rules from '../content/rules.yaml';
import { PageTitle } from '../../../packages/ui';
import { useStore } from '../store';
import { campaignPolicy, updateCampaignPolicy } from '../../../packages/domain/campaign-policy.mjs';
import { CampaignPolicySummary } from '../components/CampaignPolicySummary';
import { CampaignPolicyEditor } from '../components/CampaignPolicyEditor';
export default function Rules() {
  const { state, tenant, role, update } = useStore();
  const policy = campaignPolicy(state, tenant);
  return <><PageTitle eyebrow="Growth / Rules" title="Clear rules for every launch">Review channel defaults and set your merchant's discount limit.</PageTitle>
    <CampaignPolicySummary maxDiscount={policy.maxDiscount} channels={rules.channels} />
    {role === 'admin' && <CampaignPolicyEditor maxDiscount={policy.maxDiscount} onSave={limit => update(updateCampaignPolicy(state, tenant, limit))} />}
  </>;
}
