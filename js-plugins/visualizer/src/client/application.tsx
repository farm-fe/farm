import {
  ApplicationProvider,
  setupTheme,
  useApplicationContext
} from './context';
import { NavHead } from './components/nav-head/nav-head';
import { useEffect } from 'react';
import { Inspect } from './components/inspect';
import { colors } from './themes/color.stylex';
import { Analysis } from './components/analysis';
import './stylex.css';
import './reset.css';

function Content() {
  const { controlMode } = useApplicationContext();
  return (
    <div
      id='container'
      stylex={{
        height: 'calc(100vh - 64px)'
      }}
    >
      {controlMode === 'inspect' ? <Inspect /> : <Analysis />}
    </div>
  );
}

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
          position: 'relative',
          backgroundColor: colors.background,
          color: colors.foreground
        }}
      >
        <NavHead />
        <Content />
      </div>
    </ApplicationProvider>
  );
}
