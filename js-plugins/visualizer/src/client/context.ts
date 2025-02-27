import * as stylex from '@stylexjs/stylex';
import { createContextState } from 'foxact/context-state';
import { useCallback } from 'react';
import { darkTheme, lightTheme } from './themes/color.stylex';

const CONSTANTS = {
  theme: 'visualizer-color-scheme'
};

export type Theme = 'light' | 'dark' | 'auto';

export type ControlMode = 'inspect' | 'analysis';

export interface ApplicationConfig {
  theme: Theme;
  controlMode: ControlMode;
}

const defaultApplicationConfig: ApplicationConfig = {
  theme: (localStorage.getItem(CONSTANTS.theme) as Theme) || 'auto',
  controlMode: 'inspect'
};

export function setupTheme(theme: Theme) {
  const isDark =
    theme === 'dark' ||
    (theme === 'auto' &&
      window.matchMedia?.('(prefers-color-scheme: dark)')?.matches);
  document.documentElement.setAttribute(
    'class',
    stylex.props(isDark ? darkTheme : lightTheme).className
  );
  localStorage.setItem(CONSTANTS.theme, theme);
}

const [ApplicationProvider, useApplicationContext, useSetApplicationContext] =
  createContextState<ApplicationConfig>(defaultApplicationConfig);

export function useToggleTheme() {
  const dispatch = useSetApplicationContext();
  return useCallback(() => {
    const currentTheme =
      (localStorage.getItem(CONSTANTS.theme) as Theme) || 'auto';
    setupTheme(currentTheme === 'dark' ? 'light' : 'dark');
  }, [dispatch]);
}

export function useSetControlMode() {
  const dispatch = useSetApplicationContext();
  return useCallback(
    (controlMode: ControlMode) => {
      dispatch((state) => ({ ...state, controlMode }));
    },
    [dispatch]
  );
}

export { ApplicationProvider, useApplicationContext };
