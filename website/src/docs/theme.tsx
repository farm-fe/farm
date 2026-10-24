import { Link } from '@farmfe/docs/client';
import type { PublicSidebarItem, ThemeProps } from '@farmfe/docs/theme';
import { ColorModeControl, DocPage as DefaultDocPage, Sidebar as DefaultSidebar, filterSidebarSection, LanguageIcon, MobileNavigation, NavigationIconLink, NavigationMenu, Search, SearchProvider, sidebarSection } from '@farmfe/docs-theme-default';
import { GitHubLogoIcon } from '@radix-ui/react-icons';
import { createContext, useContext, useEffect, useState } from 'react';
import presentation from '../../farm-docs.presentation.json';
import { WebsiteSearch } from './search';

export { default as Footer } from '../theme/Footer';

const Mobile = createContext(false);
const languageLabels: Record<string, string> = { en: 'English', zh: '简体中文' };
const chineseNavigation: Record<string, string> = { Ecosystem: '生态', Guides: '指南', Config: '配置', Plugins: '插件', Team: '团队', Blog: '日志' };
function versionLabel(version: string, locale: string) {
  return version === 'current' ? (locale === 'zh' ? '2.x（最新）' : '2.x (Latest)') : version;
}

function homeTarget({ data, page }: ThemeProps) {
  const home = data.pages.find(candidate => candidate.routePath === data.home);
  return home?.alternates?.find(candidate => candidate.locale === page.locale)?.routePath ?? data.home;
}

function activeSection({ page, data }: ThemeProps) {
  const sidebar = data.sidebars.find(item => item.collection === page.collection);
  const findId = (items: PublicSidebarItem[]): string | undefined => {
    for (const item of items) {
      if (item.type === 'doc' && item.key === page.key) return item.id;
      if (item.type === 'category') { const found = findId(item.items); if (found) return found; }
    }
  };
  return sidebarSection(findId(sidebar?.items ?? []) ?? '', presentation.navigation.sections, presentation.navigation.defaultSection);
}
function MainNavigation(props: ThemeProps) {
  const { site, page, data } = props;
  const active = activeSection(props);
  const community = presentation.navigation.community[page.locale === 'zh' ? 'zh' : 'en'];
  return <nav className="farm-site-links farm-docs-navbar-links" aria-label="Main navigation">{site.theme.navbar.filter(link => presentation.navigation.primary.includes(link.label)).map(link => {
    const source = data.pages.find(candidate => candidate.routePath === link.to);
    const version = source?.version == null ? null : page.version ?? site.docs.defaultVersion;
    const alternate = source?.alternates?.find(candidate => candidate.locale === page.locale && (candidate.version ?? null) === version);
    const label = page.locale === 'zh' ? chineseNavigation[link.label] ?? link.label : link.label;
    return <Link key={link.to} to={alternate?.routePath ?? link.to} aria-current={page.kind === 'doc' && active === link.label ? 'page' : undefined}>{label}</Link>;
  })}<NavigationMenu hover={false} label={community.label} name={community.label}>{community.items.map(item => item.href.startsWith('https:') ? <a key={item.href} href={item.href} target="_blank" rel="noopener noreferrer">{item.label}</a> : <Link key={item.href} to={item.href}>{item.label}</Link>)}</NavigationMenu></nav>;
}

function RepositoryLink({ site }: ThemeProps) {
  const repository = site.theme.navbar.find(link => link.label === 'GitHub');
  return repository ? <NavigationIconLink href={repository.to} label="GitHub repository"><GitHubLogoIcon aria-hidden="true" /></NavigationIconLink> : null;
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
  const home = props.page.kind === 'page' && props.page.routePath === homeTarget(props);
  return <SearchProvider.Provider value={WebsiteSearch}><Mobile.Provider value={mobile}>
    <div className={`farm-docs-theme farm-site-shell${home ? ' farm-site-shell--home' : ''}${mobile ? ' farm-site-shell--mobile' : ''}`}>
      <a className="farm-docs-skip" href="#farm-docs-content">{props.page.locale === 'zh' ? '跳到主要内容' : 'Skip to main content'}</a>
      <slots.Navbar {...props} />
      <div className="farm-docs-layout">
        <div className="farm-docs-sidebar-slot">{!mobile && <slots.Sidebar {...props} />}</div>
        <Page {...props} />
        {!mobile && <slots.Toc {...props} />}
      </div>
      <slots.Footer {...props} />
    </div>
  </Mobile.Provider></SearchProvider.Provider>;
}

function Alternates({ site, page, data }: ThemeProps) {
  const alternates = page.alternates ?? [];
  const languages = site.locales.items.flatMap(locale => alternates.filter(item => item.locale === locale && (item.version ?? null) === (page.version ?? null)));
  const versionPage = page.version == null ? data.pages.find(candidate => candidate.routePath === presentation.navigation.versionEntry[page.locale === 'zh' ? 'zh' : 'en']) : page;
  const versions = (versionPage?.alternates ?? []).filter(item => item.locale === page.locale);
  return <>{([['Versions', versions], ['Languages', languages]] as const).map(([kind, items]) => items.length > 1 ? <NavigationMenu hover className="farm-site-alternate" key={kind} name={kind} label={<>{kind === 'Languages' && <LanguageIcon />}{kind === 'Languages' ? languageLabels[page.locale] ?? page.locale : versionLabel(page.version ?? site.docs.defaultVersion, page.locale)}</>}>{items.map(item => <Link key={item.collection} to={item.routePath} lang={kind === 'Languages' ? item.locale : undefined} aria-current={item.collection === page.collection ? 'page' : undefined}>
      {kind === 'Languages' ? languageLabels[item.locale] ?? item.locale : versionLabel(item.version ?? site.docs.defaultVersion, page.locale)}{item.missing ? ' (page unavailable)' : ''}
    </Link>)}
  </NavigationMenu> : null)}</>;
}

export function Navbar(props: ThemeProps) {
  const mobile = useContext(Mobile);
  const { site, data } = props;
  return <header className="farm-docs-navbar farm-site-navbar">
    {mobile && <div className="farm-site-menu"><MobileNavigation {...props} navigation={<>
      <MainNavigation {...props} />
      <Alternates {...props} />
      <RepositoryLink {...props} />
    </>} /></div>}
    <Link className="farm-docs-brand" to={homeTarget(props)}>{site.theme.logo && <img src={site.theme.logo.src} alt={site.theme.logo.alt} />}{site.title}</Link>
    <MainNavigation {...props} />
    {!mobile && <div className="farm-site-controls farm-docs-controls"><Alternates {...props} /><RepositoryLink {...props} /><ColorModeControl /></div>}
    <Search site={site} />
  </header>;
}

export function Sidebar(props: ThemeProps) {
  const locale = props.page.locale === 'zh' ? 'zh' : 'en';
  const sidebar = props.data.sidebars.find(item => item.collection === props.page.collection);
  const items = filterSidebarSection(sidebar?.items ?? [], presentation.navigation.sections, activeSection(props), presentation.navigation.defaultSection);
  return <DefaultSidebar {...props} items={items} expandedCategories={presentation[locale].expandedCategories} />;
}
export function DocPage(props: ThemeProps) {
  const locale = props.page.locale === 'zh' ? 'zh' : 'en';
  return <DefaultDocPage {...props} data={{ ...props.data, home: homeTarget(props) }} documentActionsLabels={locale === 'zh' ? { region: '文档格式', copy: '复制 Markdown', copying: '正在复制…', copied: 'Markdown 已复制', copyFailed: '无法复制 Markdown，请打开 Markdown 链接查看。', view: '查看 Markdown', guide: 'Agent 指南' } : undefined} homeLabel={presentation[locale].homeLabel} tocLabel={presentation[locale].tocLabel} versionLabel={versionLabel(props.page.version ?? props.site.docs.defaultVersion, props.page.locale)} />;
}
