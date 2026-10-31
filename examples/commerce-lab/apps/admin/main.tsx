import React, { lazy, Suspense, useState, useEffect } from 'react';
import '@arco-design/web-react/es/style/index.css';
import { createRoot } from 'react-dom/client';
import { Provider, useStore } from './store';
import Brand from './assets/brand.svg';
import './styles/workspace.scss';
import './styles/utilities.css';

const pages = {
  overview: lazy(() => import('./pages/Overview')),
  products: lazy(() => import('./pages/Products')),
  orders: lazy(() => import('./pages/Orders')),
  import: lazy(() => import('./pages/Import')),
  campaigns: lazy(() => import('./pages/Campaigns')),
  help: lazy(() => import('./pages/Help')),
  components: lazy(() => import('./pages/Components')),
  inventory: lazy(() => import('./pages/Inventory')),
  customers: lazy(() => import('./pages/Customers')),
  channels: lazy(() => import('./pages/Channels')),
  rules: lazy(() => import('./pages/Rules')),
  connection: lazy(() => import('./pages/Connection')),
};
const navigation = [
  ['overview', 'Overview', '01'],
  ['products', 'Products', '02'],
  ['orders', 'Orders', '03'],
  ['import', 'Data import', '04'],
  ['campaigns', 'Campaigns', '05'],
  ['help', 'Help center', '06'],
  ['components', 'Components', '07'],
  ['inventory', 'Inventory', '08'],
  ['customers', 'Customers', '09'],
  ['channels', 'Channels', '10'],
  ['rules', 'Campaign rules', '11'],
  ['connection', 'Connection', '12'],
];
class Boundary extends React.Component<
  { children: React.ReactNode },
  { error: string }
> {
  state = { error: '' };
  static getDerivedStateFromError(error: Error) {
    return { error: error.message };
  }
  render() {
    return this.state.error ? (
      <div role="alert" className="panel">
        <h1>Workspace could not load</h1>
        <p>{this.state.error}</p>
        <button onClick={() => location.reload()}>Retry</button>
      </div>
    ) : (
      this.props.children
    );
  }
}

function Workspace() {
  const { state, tenant, setTenant, role, setRole, reset } = useStore();
  const [route, setRoute] = useState(
    () => location.hash.slice(2) || 'overview',
  );
  const [theme, setTheme] = useState('light');
  useEffect(() => {
    const update = () => setRoute(location.hash.slice(2) || 'overview');
    window.addEventListener('hashchange', update);
    return () => window.removeEventListener('hashchange', update);
  }, []);
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);
  const Page = pages[route as keyof typeof pages];
  const denied = role === 'viewer' && ['import', 'campaigns'].includes(route);
  return (
    <div className="workspace" data-testid="workspace-ready">
      <aside className="sidebar">
        <a className="brand" href="#/overview">
          <Brand width={32} height={32} />
          <span>
            commerce<span className="brand-sub">LAB / FARM V2</span>
          </span>
        </a>
        <span className="nav-heading">WORKSPACE</span>
        <nav>
          {navigation.map(([id, label, number]) => (
            <a
              key={id}
              href={`#/${id}`}
              className={route === id ? 'active' : ''}
            >
              <span aria-hidden="true">{number}</span>
              {label}
            </a>
          ))}
        </nav>
        <div className="sidebar-note">
          <span className="live-dot" />
          Local demo · fixed seed
          <p>
            One business application.
            <br />
            Three plugin pathways.
          </p>
        </div>
      </aside>
      <div className="main-shell">
        <header className="topbar">
          <label>
            Merchant
            <select
              aria-label="Merchant"
              value={tenant}
              onChange={(e) => setTenant(e.target.value)}
            >
              {state.tenants.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name}
                </option>
              ))}
            </select>
          </label>
          <div className="top-actions">
            <span className="group-tag">{__COMMERCE_GROUP__} pipeline</span>
            <label>
              Role
              <select
                aria-label="Role"
                value={role}
                onChange={(e) => setRole(e.target.value)}
              >
                <option value="admin">Admin</option>
                <option value="viewer">Viewer</option>
              </select>
            </label>
            <button
              onClick={() => setTheme(theme === 'light' ? 'dark' : 'light')}
            >
              Toggle theme
            </button>
            <button onClick={reset}>Reset demo</button>
          </div>
        </header>
        <main>
          <Boundary key={`${route}-${tenant}-${role}`}>
            <Suspense
              fallback={<div className="panel">Loading workspace…</div>}
            >
              {denied ? (
                <div className="panel" role="alert">
                  Your role cannot access this workspace.
                </div>
              ) : Page ? (
                <Page />
              ) : (
                <div className="panel">
                  Workspace not found.{' '}
                  <a href="#/overview">Return to overview</a>
                </div>
              )}
            </Suspense>
          </Boundary>
        </main>
        <footer>
          Commerce Lab{' '}
          <span>
            Deterministic business fixtures · Farm v2 ecosystem comparison
          </span>
        </footer>
      </div>
    </div>
  );
}
createRoot(document.getElementById('root')!).render(
  <Provider>
    <Workspace />
  </Provider>,
);
