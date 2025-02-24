import {
  ApplicationProvider,
  setupTheme,
  useApplicationContext
} from './context';
import { NavHead } from './components/nav-head/nav-head';
import './stylex.css';
import './reset.css';
import { useEffect } from 'react';
import { Inspect } from './components/inspect';

export function App() {
  const { theme } = useApplicationContext();

  useEffect(() => {
    setupTheme(theme);
  }, []);

  return (
    <ApplicationProvider key='app'>
      <div
        stylex={{
          height: '100%',
          width: '100%',
          position: 'relative'
        }}
      >
        <NavHead />
        <div id='container' style={{ height: 'calc(100vh - 64px)' }}>
          <Inspect />
        </div>
      </div>
    </ApplicationProvider>
  );
}
