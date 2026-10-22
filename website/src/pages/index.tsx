import { Link, useLocale } from '@farmfe/docs/client';
import React from 'react';
import { SiteText } from '../site-text';

export const metadata = {
  title: 'Farm',
  description: 'A Rust-powered build tool for modern web projects.'
};

const configuration = `import { defineConfig } from '@farmfe/core';
import react from '@farmfe/plugin-react';

export default defineConfig({
  plugins: [react()],
});`;

export default function Home() {
  const prefix = useLocale() === 'zh' ? '/zh/docs/' : '/docs/';
  return <div className="farm-editorial-home">
    <section className="farm-editorial-hero" aria-labelledby="farm-home-title">
      <div>
        <p className="farm-editorial-eyebrow"><SiteText id="editorial.eyebrow" /></p>
        <h1 id="farm-home-title"><SiteText id="editorial.title" /><br /><em><SiteText id="editorial.titleAccent" /></em></h1>
        <p className="farm-editorial-intro"><SiteText id="editorial.intro" /></p>
        <div className="farm-editorial-actions">
          <Link className="farm-editorial-primary" to={`${prefix}quick-start`}><SiteText id="home.quickStart" /><span aria-hidden="true">→</span></Link>
          <a className="farm-editorial-secondary" href="https://github.com/farm-fe/farm">GitHub <span aria-hidden="true">↗</span></a>
        </div>
        <p className="farm-editorial-install"><span aria-hidden="true">$ </span><code>npm create farm@latest</code></p>
      </div>
      <figure className="farm-editorial-config">
        <figcaption><span>farm.config.ts</span><span aria-hidden="true">● ● ●</span></figcaption>
        <pre><code>{configuration}</code></pre>
        <ol className="farm-editorial-pipeline" aria-label={useLocale() === 'zh' ? '构建流程' : 'Build pipeline'}>
          <li><SiteText id="editorial.source" /></li><li>Rust core</li><li><SiteText id="editorial.output" /></li>
        </ol>
        <p className="farm-editorial-caption"><SiteText id="editorial.configCaption" /></p>
      </figure>
    </section>
    <section className="farm-editorial-features" aria-labelledby="farm-features-title">
      <div className="farm-editorial-section-title"><h2 id="farm-features-title"><SiteText id="editorial.features" /></h2><p><SiteText id="editorial.featuresIntro" /></p></div>
      <div className="farm-editorial-cards">
        <Link to={`${prefix}why-farm`}><span className="farm-editorial-number">01 / CORE</span><h3><SiteText id="editorial.coreTitle" /></h3><p><SiteText id="editorial.coreDescription" /></p></Link>
        <Link to={`${prefix}features/dev-server`}><span className="farm-editorial-number">02 / WORKFLOW</span><h3><SiteText id="editorial.workflowTitle" /></h3><p><SiteText id="editorial.workflowDescription" /></p></Link>
        <Link to={`${prefix}plugins/official-plugins/overview`}><span className="farm-editorial-number">03 / ECOSYSTEM</span><h3><SiteText id="editorial.pluginsTitle" /></h3><p><SiteText id="editorial.pluginsDescription" /></p></Link>
      </div>
    </section>
    <section className="farm-editorial-community"><div><h2><SiteText id="editorial.communityTitle" /></h2><p><SiteText id="editorial.communityDescription" /></p></div><Link to={`${prefix}contribution`}><SiteText id="home.contribute" /><span aria-hidden="true"> →</span></Link></section>
  </div>;
}
