import { Link, useColorMode, useLocale } from '@farmfe/docs/client';
import type { NotFoundProps, PublicSidebarItem, ThemeProps } from '@farmfe/docs/theme';
import type { ReactNode } from 'react';
import { useEffect, useId, useRef, useState } from 'react';
import { CodeBlock } from './code-block.js';
import { DiagramViewer } from './diagram-viewer.js';
import type { DocumentActionsLabels } from './document-actions.js';
import { Search } from './search.js';

export { CodeBlock, type CodeBlockProps } from './code-block.js';
export { DocumentActions, type DocumentActionsLabels, type DocumentActionsProps } from './document-actions.js';
export { Search, type SearchProps, SearchProvider } from './search.js';
export { SearchPage, type SearchPageLabels, type SearchPageProps } from './search-page.js';

function useMobileNavigation() {
  const [mobile, setMobile] = useState(false);
  useEffect(() => {
    let media: MediaQueryList;
    try { media = matchMedia('(max-width: 996px)'); } catch { return; }
    const update = () => setMobile(media.matches);
    update();
    media.addEventListener('change', update);
    return () => media.removeEventListener('change', update);
  }, []);
  return mobile;
}
export function ColorModeControl() {
  const { colorMode, setColorMode } = useColorMode();
  const chinese = useLocale().startsWith('zh');
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  const next = colorMode === 'light' ? 'dark' : colorMode === 'dark' ? 'system' : 'light';
  const names = chinese ? { light: '浅色', dark: '深色', system: '跟随系统' } : { light: 'Light', dark: 'Dark', system: 'Follow system' };
  const label = chinese ? `当前主题：${names[colorMode]}。点击切换为${names[next]}。` : `Current theme: ${names[colorMode]}. Click to switch to ${names[next]}.`;
  return mounted ? <button className="farm-docs-color-control farm-docs-color-button" type="button" aria-label={label} title={label} data-color-mode={colorMode} onClick={() => setColorMode(next)}>
    <svg aria-hidden="true" focusable="false" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">
      {colorMode === 'light' ? <><circle cx="12" cy="12" r="4" /><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5m11 11L19 19M5 19l1.5-1.5m11-11L19 5" /></> : colorMode === 'dark' ? <path d="M20.5 13A8.5 8.5 0 0 1 11 3.5a8.5 8.5 0 1 0 9.5 9.5Z" /> : <><rect x="3" y="4" width="18" height="13" rx="2" /><path d="M8 21h8m-4-4v4" /></>}
    </svg>
  </button> : null;
}

export function MobileNavigation(props: ThemeProps & { navigation?: ReactNode }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  const previousOverflow = useRef<string | null>(null);
  const [open, setOpen] = useState(false);
  const id = useId();
  function releaseScroll() {
    if (previousOverflow.current !== null) {
      document.body.style.overflow = previousOverflow.current;
      previousOverflow.current = null;
    }
  }
  function dismiss() {
    dialog.current?.close();
    releaseScroll();
    setOpen(false);
  }
  useEffect(() => {
    const element = dialog.current;
    const historyNavigation = () => { if (element?.open) dismiss(); };
    window.addEventListener('popstate', historyNavigation);
    window.addEventListener('farm-docs:before-focus', historyNavigation);
    return () => {
      window.removeEventListener('popstate', historyNavigation);
      window.removeEventListener('farm-docs:before-focus', historyNavigation);
      if (element?.open) element.close();
      releaseScroll();
    };
  }, []);
  return <>
    <button className="farm-docs-menu-toggle" aria-label="Open navigation" type="button" aria-haspopup="dialog" aria-controls={id} aria-expanded={open} onClick={() => {
      const element = dialog.current;
      if (!element || element.open) return;
      element.showModal();
      previousOverflow.current = document.body.style.overflow;
      document.body.style.overflow = 'hidden';
      setOpen(true);
      closeButton.current?.focus();
    }}>Open navigation</button>
    <dialog id={id} ref={dialog} aria-label="Navigation" className="farm-docs-mobile-dialog" onClose={() => { releaseScroll(); setOpen(false); }} onCancel={event => { event.preventDefault(); dismiss(); }} onKeyDown={event => {
      if (event.key !== 'Tab') return;
      const stops = Array.from(event.currentTarget.querySelectorAll<HTMLElement>('a[href], area[href], button, input, select, textarea, summary, [tabindex], [contenteditable]'))
        .filter(element => (element.tabIndex >= 0 || (element.isContentEditable && !element.hasAttribute('tabindex'))) && !element.matches(':disabled') && !element.closest('[inert]') && element.getClientRects().length > 0 && getComputedStyle(element).visibility === 'visible')
        .sort((left, right) => (left.tabIndex > 0 ? left.tabIndex : Infinity) - (right.tabIndex > 0 ? right.tabIndex : Infinity));
      const first = stops[0], last = stops[stops.length - 1];
      if (first && last && ((event.shiftKey && document.activeElement === first) || (!event.shiftKey && document.activeElement === last))) {
        event.preventDefault();
        (event.shiftKey ? last : first).focus();
      }
    }} onClick={event => {
      if (event.target === event.currentTarget) {
        const bounds = event.currentTarget.getBoundingClientRect();
        if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) dismiss();
        return;
      }
      if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      const anchor = event.target instanceof Element ? event.target.closest('a[href]') : null;
      // Release native modal focus before the existing document-level router acts.
      if (anchor && !anchor.hasAttribute('download') && anchor.getAttribute('target') !== '_blank') dismiss();
    }}>
      <button type="button" ref={closeButton} onClick={dismiss}>Close navigation</button>
      <ColorModeControl />
      {props.navigation}
      <props.components.Sidebar {...props} />
      <props.components.Toc {...props} />
    </dialog>
  </>;
}

export function Layout(props: ThemeProps) {
  const mobile = useMobileNavigation();
  const slots = props.components;
  const Page = props.page.kind === 'blog' || props.page.blogView ? slots.BlogPage : slots.DocPage;
  return <div className="farm-docs-theme">
    <a className="farm-docs-skip" href="#farm-docs-content">Skip to content</a>
    <slots.Navbar {...props} />
    <div className="farm-docs-layout">
      <div className="farm-docs-sidebar-slot">{!mobile && <slots.Sidebar {...props} />}</div>
      <Page {...props} />
      {!mobile && <slots.Toc {...props} />}
    </div>
    <slots.Footer {...props} />
  </div>;
}
function AlternateNavigation({ site, data, page }: ThemeProps) {
  const alternates = page.alternates ?? [];
  const languages = site.locales.items.flatMap(locale => alternates.filter(alternate => alternate.locale === locale && (alternate.version ?? null) === (page.version ?? null)));
  const versions = alternates.filter(alternate => alternate.locale === page.locale);
  return <>{([['Languages', languages], ['Versions', versions]] as const).map(([kind, items]) => items.length > 1 ? <nav key={kind} aria-label={kind} className="farm-docs-alternates"><ul>{items.map(alternate => {
    const label = kind === 'Languages' ? alternate.locale : alternate.version ?? site.docs.defaultVersion;
    const fallback = alternate.key ? data.pages.find(target => target.key === alternate.key)?.fallbackSource : undefined;
    return <li key={alternate.collection}><Link to={alternate.routePath} aria-current={alternate.collection === page.collection ? 'page' : undefined}>{label}{alternate.missing ? ` (page unavailable; ${kind === 'Versions' ? 'version' : 'language'} home)` : fallback ? ` (content from ${fallback.locale})` : ''}</Link></li>;
  })}</ul></nav> : null)}</>;
}
export function Navbar(props: ThemeProps) {
  const { site, data } = props;
  const mobile = useMobileNavigation();
  const links = <nav className="farm-docs-navbar-links" aria-label="Main navigation">{site.theme.navbar.map(link => <Link key={link.to} to={link.to}>{link.label}</Link>)}</nav>;
  return <header className="farm-docs-navbar">
    {mobile && <MobileNavigation {...props} navigation={<>{links}<AlternateNavigation {...props} /></>} />}
    <Link className="farm-docs-brand" to={data.home}>{site.theme.logo && <img src={site.theme.logo.src} alt={site.theme.logo.alt} />}{site.title}</Link>
    {links}
    <div className="farm-docs-controls"><AlternateNavigation {...props} /><ColorModeControl /></div>
    <Search site={site} />
  </header>;
}

function SidebarItems({ items, active, expanded }: { items: PublicSidebarItem[]; active: string; expanded: readonly string[] }) {
  const containsActive = (items: PublicSidebarItem[]): boolean => items.some(item => item.type === 'doc' ? item.key === active : item.type === 'category' && containsActive(item.items));
  return <ul>{items.map((item, index) => <li key={item.type === 'doc' ? item.key : `${item.type}:${index}`}>
    {item.type === 'category' ? <details key={`${active}:${item.label}`} open={expanded.includes(item.label) || containsActive(item.items)}><summary><svg className="farm-docs-sidebar-chevron" aria-hidden="true" focusable="false" viewBox="0 0 16 16" fill="none"><path d="m6 4 4 4-4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" /></svg><span className="farm-docs-sidebar-label">{item.label}</span></summary><SidebarItems items={item.items} active={active} expanded={expanded} /></details> : <Link to={item.href} aria-current={item.type === 'doc' && item.key === active ? 'page' : undefined}>{item.label}</Link>}
  </li>)}</ul>;
}
export function Sidebar({ data, page, expandedCategories = [], items }: ThemeProps & { expandedCategories?: readonly string[]; items?: PublicSidebarItem[] }) {
  const sidebar = data.sidebars.find(item => item.collection === page.collection);
  const visible = items ?? sidebar?.items ?? [];
  return visible.length ? <nav className="farm-docs-sidebar" aria-label="Documentation"><SidebarItems items={visible} active={page.key} expanded={expandedCategories} /></nav> : null;
}
function PageContent({ page, children, beforeArticle }: { page: ThemeProps['page']; children: ReactNode; beforeArticle?: ReactNode }) {
  return <main id="farm-docs-content" tabIndex={-1}>
    {page.fallbackSource && <aside className="farm-docs-fallback" aria-label="Translation fallback">This page uses {page.fallbackSource.locale} content. <Link to={page.fallbackSource.routePath}>View source language</Link></aside>}
    {beforeArticle}
    <article>{children}</article>
    <DiagramViewer />
    {(page.previous || page.next) && <nav aria-label="Previous and next pages" className="farm-docs-pagination">{page.previous && <Link to={page.previous.href} aria-label={`Previous: ${page.previous.label}`}>Previous: {page.previous.label}</Link>}{page.next && <Link to={page.next.href} aria-label={`Next: ${page.next.label}`}>Next: {page.next.label}</Link>}</nav>}
  </main>;
}
function sidebarAncestors(items: PublicSidebarItem[], key: string, ancestors: string[] = []): string[] | null {
  for (const item of items) {
    if (item.type === 'doc' && item.key === key) return ancestors;
    if (item.type === 'category') {
      const found = sidebarAncestors(item.items, key, [...ancestors, item.label]);
      if (found) return found;
    }
  }
  return null;
}
export function DocPage(props: ThemeProps & { homeLabel?: string; versionLabel?: string | null; tocLabel?: string; documentActionsLabels?: Partial<DocumentActionsLabels>; agentGuideUrl?: string | null }) {
  const { page, children, data, homeLabel = 'Home', versionLabel = page.version, tocLabel = 'On this page' } = props;
  const ancestors = sidebarAncestors(data.sidebars.find(sidebar => sidebar.collection === page.collection)?.items ?? [], page.key) ?? [];
  const metadata = page.kind === 'doc' ? <>
    <nav className="farm-docs-breadcrumbs" aria-label="Breadcrumbs"><Link to={data.home}>{homeLabel}</Link><span aria-hidden="true">›</span>{ancestors.flatMap((label, index) => [<span key={`label:${index}`}>{label}</span>, <span key={`separator:${index}`} aria-hidden="true">›</span>])}<span>{page.title}</span></nav>
    {versionLabel && <span className="farm-docs-version">Version: {versionLabel}</span>}
    {page.headings.length > 0 && <details className="farm-docs-mobile-toc"><summary>{tocLabel}</summary><Toc {...props} /></details>}
  </> : null;
  return <PageContent page={page} beforeArticle={metadata}>{children}</PageContent>;
}
export function BlogPage(props: ThemeProps) {
  const { page, data } = props;
  if (page.blogView?.fullContent && !props.listingPosts) throw new Error('[DOCS_BLOG_CONTENT] full listing articles were not loaded');
  const post = page.blog;
  const views = ['blog-index', 'blog-archive', 'blog-tags'].flatMap(kind => {
    const target = data.pages.find(candidate => candidate.collection === page.collection && candidate.blogView?.kind === kind);
    return target ? [target] : [];
  });
  return <PageContent page={page} beforeArticle={<>
    {views.length > 0 && <nav aria-label="Blog navigation"><ul>{views.map(target => <li key={target.key}><Link to={target.routePath} aria-current={target.key === page.key ? 'page' : undefined}>{target.title}</Link></li>)}</ul></nav>}
    {post && (post.date || post.authors.length > 0 || post.tags.length > 0) && <section aria-label="Post information" className="farm-docs-post-information">
      {post.date && <time dateTime={post.publishedAt ?? post.date}>{post.date}</time>}
      {post.authors.length > 0 && <ul className="farm-docs-authors">{post.authors.map((author, index) => <li key={`${index}:${author.name}`}>
        {author.image && <img src={author.image} alt={author.name} width={48} height={48} />}
        {author.url ? <Link to={author.url}>{author.name}</Link> : <span>{author.name}</span>}
      </li>)}</ul>}
      {post.tags.length > 0 && <nav aria-label="Post tags"><ul>{post.tags.map(tag => {
        const target = data.pages.find(candidate => candidate.collection === page.collection && candidate.blogView?.kind === 'blog-tag' && candidate.blogView.tag === tag.id);
        if (!target) throw new Error(`[DOCS_THEME] Missing blog tag page: ${page.collection}/${tag.id}`);
        return <li key={tag.id}><Link to={target.routePath}>{tag.label}</Link></li>;
      })}</ul></nav>}
    </section>}
  </>}>{page.blogView?.fullContent && props.listingPosts?.length ? <>
    <h1>{page.title}</h1>
    {props.listingPosts?.map(post => <section key={post.page.key} className="farm-docs-blog-post">
      <Link to={post.page.routePath}>{post.page.title}</Link>
      {post.page.blog?.date && <time dateTime={post.page.blog.publishedAt ?? post.page.blog.date}>{post.page.blog.date}</time>}
      {post.children}
    </section>)}
  </> : props.children}</PageContent>;
}
export function Toc({ page }: ThemeProps) {
  return <aside className="farm-docs-toc" aria-label="On this page"><ul>{page.headings.filter(heading => heading.depth > 1).map(heading => <li key={heading.id}><a href={`#${heading.id}`}>{heading.title}</a></li>)}</ul></aside>;
}
export function Footer({ site }: ThemeProps) {
  return <footer>{site.theme.footer.links.map(link => <Link key={link.to} to={link.to}>{link.label}</Link>)}{site.theme.footer.copyright && <p>{site.theme.footer.copyright}</p>}</footer>;
}
export function NotFound({ data }: NotFoundProps) {
  return <main id="farm-docs-content" tabIndex={-1}><h1>{data.notFound.heading}</h1><p>{data.notFound.description}</p><Link to={data.notFound.home}>{data.notFound.homeLabel}</Link></main>;
}
export const components = { pre: CodeBlock };

export { LanguageIcon, NavigationIconLink, NavigationMenu } from './navigation.js';

export { filterSidebarSection, sidebarSection } from './sidebar-section.js';
