import React, { useState } from 'react';
import rules from '../content/rules.yaml';
import channelConfiguration from 'commerce-channel-capabilities.js';
import LaunchIcon from '~icons/commerce/launch';
import { publishWithDiagnostics } from '../../../packages/domain/publication-diagnostics.mjs';
import { useStore } from '../store';
import {
  queryProducts,
} from '../../../packages/domain/index.mjs';
import { PageTitle, Panel, Badge } from '../../../packages/ui';
import { quoteCampaign } from '../../../packages/sdk/src';
import { CampaignProductPicker } from '../components/CampaignProductPicker';
import { campaignPolicy } from '../../../packages/domain/campaign-policy.mjs';
import { CampaignPricePreview } from '../components/CampaignPricePreview';
export default function Campaigns() {
  const { state, update, tenant } = useStore();
  const policy = campaignPolicy(state, tenant);
  const [name, setName] = useState('');
  const [discount, setDiscount] = useState(10);
  const [error, setError] = useState('');
  const [selected, setSelected] = useState<string[]>([]);
  const [channel, setChannel] = useState(channelConfiguration.channels[0]);
  const products = queryProducts(state, tenant).filter(
    (p) => p.status === 'published',
  );
  const quote = Number.isInteger(discount) && discount > 0 && discount <= policy.maxDiscount
    ? quoteCampaign(tenant, products.filter(product => selected.includes(product.id)), discount)
    : null;
  return (
    <>
      <PageTitle
        eyebrow="Growth / Campaigns"
        title="Make the next launch count"
      >
        Choose a collection, set an offer, and publish.
      </PageTitle>
      <div className="grid-two">
        <Panel title="New campaign">
          <form
            onSubmit={(e) => {
              e.preventDefault();
              try {
                update(
                  publishWithDiagnostics(state, tenant, {
                    name,
                    discount,
                    productIds: selected,
                  }, channel, channelConfiguration),
                );
                setName('');
                setSelected([]);
                setError('');
              } catch (err) {
                setError(String(err));
              }
            }}
          >
            <label>
              Campaign name
              <input
                required
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </label>
            <label>
              Discount percent
              <input
                type="number"
                min="1"
                max={policy.maxDiscount}
                value={discount}
                onChange={(e) => setDiscount(Number(e.target.value))}
              />
            </label>
            <p>
              {products.length} published products · {rules.channels.join(', ')}
            </p>
            <label>
              Publication channel
              <select aria-label="Publication channel" value={channel} onChange={event => setChannel(event.target.value)}>
                {channelConfiguration.channels.map(value => <option key={value}>{value}</option>)}
              </select>
            </label>
            <p data-testid="channel-rule-version">Channel rules: {channelConfiguration.version}</p>
            <CampaignProductPicker products={products} selected={selected} onChange={setSelected} />
            <div role="region" aria-label="Campaign price preview">
              {quote ? <CampaignPricePreview key={selected.join(',')} quote={quote} products={products} /> : <p>Enter a whole discount from 1 to {policy.maxDiscount} to preview prices.</p>}
              {!selected.length && <p>Select published products to preview this collection.</p>}
            </div>
            {error && <p role="alert">{error}</p>}
            <button className="primary" type="submit">
              <LaunchIcon data-testid="launch-icon" width={22} height={22} aria-hidden="true" />
              Publish campaign
            </button>
          </form>
        </Panel>
        <Panel title="Published campaigns">
          {state.campaigns
            .filter((c) => c.tenantId === tenant)
            .map((c) => (
              <article className="campaign" key={c.id}>
                <h3>{c.name}</h3>
                <p>
                  {c.discount}% off · {c.productIds.length} products
                </p>
                <Badge>{c.status}</Badge>
                {'channel' in c && <p>Published to {String(c.channel)}</p>}
              </article>
            ))}
        </Panel>
      </div>
    </>
  );
}
