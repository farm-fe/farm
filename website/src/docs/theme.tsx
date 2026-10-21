import { Link } from '@farmfe/docs/client';
import type { ThemeProps } from '@farmfe/docs/theme';
import { ColorModeControl, MobileNavigation, Search } from '@farmfe/docs-theme-default';
import { createContext, useContext, useEffect, useState } from 'react';

export { default as Footer } from '../theme/Footer';

const Mobile = createContext(false);
const languageLabels: Record<string, string> = { en: 'English', zh: '简体中文' };
const chineseNavigation: Record<string, string> = { Guides: '指南', Config: '配置', Plugins: '插件', Team: '团队', Blog: '日志' };
function versionLabel(version: string, locale: string) {
  return version === 'current' ? (locale === 'zh' ? '2.x（最新）' : '2.x (Latest)') : version;
}

function MainNavigation({ site, page, data }: ThemeProps) {
  return <nav className="farm-site-links" aria-label="Main navigation">{site.theme.navbar.map(link => {
    const source = data.pages.find(candidate => candidate.routePath === link.to);
    const version = source?.version == null ? null : page.version ?? site.docs.defaultVersion;
    // Choose an existing Rust target; never derive a locale/version URL here.
    const alternate = source?.alternates?.find(candidate => candidate.locale === page.locale && (candidate.version ?? null) === version);
    return <Link key={link.to} to={alternate?.routePath ?? link.to}>{page.locale === 'zh' ? chineseNavigation[link.label] ?? link.label : link.label}</Link>;
  })}</nav>;
}

/** Adapt the public theme slots to the original website shell. */
export function Layout(props: ThemeProps) {
  const [mobile, setMobile] = useState(false);
  useEffect(() => {
    const media = matchMedia('(max-width: 996px)');
    const update = () => setMobile(media.matches);
    update();
    media.addEventListener('change', update);
    return () => media.removeEventListener('change', update);
  }, []);
  const slots = props.components;
  const Page = props.page.kind === 'blog' || props.page.blogView ? slots.BlogPage : slots.DocPage;
  const home = props.page.kind === 'page' && props.page.routePath === props.data.home;
  return <Mobile.Provider value={mobile}>
    <div className={`farm-site-shell${home ? ' farm-site-shell--home' : ''}${mobile ? ' farm-site-shell--mobile' : ''}`}>
      <a className="farm-docs-skip" href="#farm-docs-content">Skip to content</a>
      <slots.Navbar {...props} />
      <div className="farm-docs-layout">
        <div className="farm-docs-sidebar-slot">{!mobile && <slots.Sidebar {...props} />}</div>
        <Page {...props} />
        {!mobile && <slots.Toc {...props} />}
      </div>
      <slots.Footer {...props} />
    </div>
  </Mobile.Provider>;
}

function Alternates({ site, page }: ThemeProps) {
  const alternates = page.alternates ?? [];
  const languages = site.locales.items.flatMap(locale => alternates.filter(item => item.locale === locale && (item.version ?? null) === (page.version ?? null)));
  const versions = alternates.filter(item => item.locale === page.locale);
  return <>{([['Languages', languages], ['Versions', versions]] as const).map(([kind, items]) => items.length > 1 ? <details className="farm-site-alternate" key={kind}>
    <summary>{kind === 'Languages' ? languageLabels[page.locale] ?? page.locale : versionLabel(page.version ?? site.docs.defaultVersion, page.locale)}</summary>
    <nav aria-label={kind}>{items.map(item => <Link key={item.collection} to={item.routePath} aria-current={item.collection === page.collection ? 'page' : undefined}>
      {kind === 'Languages' ? languageLabels[item.locale] ?? item.locale : versionLabel(item.version ?? site.docs.defaultVersion, page.locale)}{item.missing ? ' (page unavailable)' : ''}
    </Link>)}</nav>
  </details> : null)}</>;
}

export function Navbar(props: ThemeProps) {
  const mobile = useContext(Mobile);
  const { site, data } = props;
  return <header className="farm-docs-navbar farm-site-navbar">
    {mobile && <div className="farm-site-menu"><MobileNavigation {...props} navigation={<>
      <MainNavigation {...props} />
      <Alternates {...props} />
    </>} /></div>}
    <Link className="farm-docs-brand" to={data.home}>{site.theme.logo && <img src={site.theme.logo.src} alt={site.theme.logo.alt} />}{site.title}</Link>
    <MainNavigation {...props} />
    {!mobile && <div className="farm-site-controls"><Alternates {...props} /><ColorModeControl /></div>}
    <Search site={site} />
  </header>;
}
