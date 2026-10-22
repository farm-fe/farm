import { Link, useLocale } from '@farmfe/docs/client';
import { GitHubLogoIcon } from '@radix-ui/react-icons';
import React from 'react';
import Benchmark from '../components/Benchmark';
import BentoGridCard from '../components/MagicUi/card';
import { SiteText } from '../site-text';

export const metadata = { title: 'Farm Documentation', description: 'Description will go into a meta tag in <head />' };

export default function Home() {
  const prefix = useLocale() === 'zh' ? '/zh/docs/' : '/docs/';
  return <div className="farm-original-home">
    <div className="farm-first-hero-grid">
    <section className="farm-original-introduction" aria-labelledby="farm-home-title">
      <a className="farm-original-star" href="https://github.com/farm-fe/farm" target="_blank" rel="noopener noreferrer"><span aria-hidden="true">⭐️</span><span>Give Star with Farm Github</span><GitHubLogoIcon aria-hidden="true" /></a>
      <h1 id="farm-home-title"><span><SiteText id="home.extremely" /><em><SiteText id="home.fast" /></em><span className="farm-first-title-web"><SiteText id="home.web" /></span></span><span><em><SiteText id="home.buildTool" /></em></span><span><SiteText id="home.writtenIn" /><em><SiteText id="home.rust" /></em></span></h1>
      <p className="farm-original-description"><strong><SiteText id="home.farm" /></strong><SiteText id="home.description" /></p>
      <div className="farm-original-actions"><Link className="farm-original-primary" to={`${prefix}quick-start`}><SiteText id="home.quickStart" /></Link><Link className="farm-original-secondary" to={`${prefix}why-farm`}><SiteText id="home.whyFarm" /></Link><Link className="farm-original-secondary" to={`${prefix}contribution`}><SiteText id="home.contribute" /></Link></div>
    </section>
    <section className="farm-original-benchmark" aria-label={useLocale() === 'zh' ? '官网原有性能对比' : 'Original homepage benchmark'}>
      <Benchmark />
      <p className="farm-original-benchmark-source">{useLocale() === 'zh' ? '数据沿用线上官网及原有性能对比项目，非本次 v2 实测结果。' : 'Data retained from the live homepage and its original benchmark project; not a new v2 measurement.'}</p>
    </section>
    </div>
    <section className="farm-original-features" aria-label={useLocale() === 'zh' ? 'Farm 特性' : 'Farm features'}><BentoGridCard /></section>
  </div>;
}
