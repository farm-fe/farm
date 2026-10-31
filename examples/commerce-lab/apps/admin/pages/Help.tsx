import React, { useState } from 'react';
import Guide from '../content/import-guide.mdx';
import OrderGuide from '../content/order-guide.mdx';
import CampaignGuide from '../content/campaign-guide.mdx';
import { HelpTopics, type HelpTopic } from '../components/HelpTopics';
import { HelpSearch } from '../components/HelpSearch';
const topics: HelpTopic[] = [
  { id: 'import', title: 'Product import playbook', keywords: 'csv worker validation sku price drafts merchant' },
  { id: 'orders', title: 'Order operations', keywords: 'refund ship fulfilment complete customer revenue viewer' },
  { id: 'campaigns', title: 'Campaign launch checklist', keywords: 'campaign discount policy yaml preview published collection' },
];
const articles: Record<string, React.ComponentType> = { import: Guide, orders: OrderGuide, campaigns: CampaignGuide };
export default function Help() {
  const [search, setSearch] = useState(''); const [selected, setSelected] = useState('import');
  const normalized = search.trim().toLowerCase();
  const matches = topics.filter(topic => `${topic.title} ${topic.keywords}`.toLowerCase().includes(normalized));
  const Article = articles[selected];
  return (
    <><HelpSearch value={search} onChange={setSearch} /><HelpTopics topics={matches} selected={selected} onSelect={setSelected} />
      <section aria-label="Help article" className="panel prose"><Article /></section></>
  );
}
