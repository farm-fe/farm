import { Link, useLocale } from '@farmfe/docs/client';
import React from 'react';
import { SiteText } from '../../site-text';

export default function FooterWrapper() {
  const zh = useLocale() === 'zh';
  const prefix = zh ? '/zh/docs/' : '/docs/';
  return <footer className="farm-site-footer farm-editorial-footer">
    <div className="farm-editorial-footer-top">
      <div><Link className="farm-editorial-footer-brand" to={zh ? '/zh' : '/'}><img src="/img/logo.png" alt="" />Farm</Link><p><SiteText id="editorial.footer" /></p></div>
      <nav aria-label={zh ? '页脚导航' : 'Footer navigation'}>
        <div><h2>{zh ? '指南' : 'Guides'}</h2><Link to={`${prefix}quick-start`}><SiteText id="home.quickStart" /></Link><Link to={`${prefix}why-farm`}><SiteText id="home.whyFarm" /></Link></div>
        <div><h2>{zh ? '社区' : 'Community'}</h2><a href="https://github.com/farm-fe/farm?tab=readme-ov-file#chat-with-us">{zh ? '微信群' : 'WeChat group'}</a><a href="https://discord.com/invite/mDErq9aFnF">Discord</a></div>
        <div><h2>{zh ? '更多' : 'More'}</h2><a href="https://github.com/farm-fe/farm">GitHub</a><a href="https://x.com/FarmFe71928">X / Twitter</a></div>
      </nav>
    </div>
    <div className="farm-editorial-footer-bottom"><span>© Farm Community · Built with Farm.</span><a href="/THIRD_PARTY_NOTICES.txt">{zh ? '第三方许可声明' : 'Third-party notices'}</a></div>
  </footer>;
}
