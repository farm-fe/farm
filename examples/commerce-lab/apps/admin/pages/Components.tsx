import React from 'react';
import Brand from '../assets/brand.svg';
import { PageTitle, Panel, Badge } from '../../../packages/ui';
import { ExtensionCatalog } from '../components/ExtensionCatalog';
// Intentionally supplied by the configured auto-import implementation.
export default function Components() {
  const [clicks, setClicks] = useState(0);
  return (
    <>
      <PageTitle eyebrow="Design / Components" title="Built to work together">
        A shared vocabulary for every part of the workspace.
      </PageTitle>
      <ExtensionCatalog />
      <div className="grid-two">
        <Panel title="Interactive SVG">
          <button
            aria-label="Interactive SVG"
            onClick={() => setClicks(clicks + 1)}
          >
            <Brand width={64} height={64} />
          </button>
          <p>
            Icon clicked {clicks} time{clicks === 1 ? '' : 's'}
          </p>
        </Panel>
        <Panel title="Status language">
          <div className="toolbar">
            {['draft', 'published', 'paid', 'shipped', 'refunded'].map((s) => (
              <Badge key={s}>{s}</Badge>
            ))}
          </div>
        </Panel>
        <Panel title="Tailwind utilities">
          <div className="rounded-xl bg-emerald-100 p-6 text-emerald-900">
            Ready for your next launch
          </div>
        </Panel>
        <Panel title="Sass theme">
          <div className="swatches">
            <i />
            <i />
            <i />
          </div>
          <p>Shared variables carry the brand across the entire workspace.</p>
        </Panel>
      </div>
    </>
  );
}
