import { createTheme, defineVars } from '@stylexjs/stylex';

const preferDarkQuery = '@media (prefers-color-scheme: dark)';

export const lightPrimitiveThemes = {
  background: '#fff',
  foreground: '#000',
  accents_1: '#fafafa',
  accents_2: '#eaeaea',
  accents_3: '#999',
  accents_4: '#888',
  accents_5: '#666',
  accents_6: '#444',
  accents_7: '#333',
  accents_8: '#111',
  selection: '#79ffe1',
  secondary: '#666',
  code: '#f81ce5',
  border: '#eaeaea',
  error: '#e00',
  errorLight: '#ff1a1a',
  errorLighter: '#f7d4d6',
  errorDark: '#c50000',
  success: '#0070f3',
  successLight: '#3291ff',
  successLighter: '#d3e5ff',
  successDark: '#0761d1',
  warning: '#f5a623',
  warningLight: '#f7b955',
  warningLighter: '#ffefcf',
  warningDark: '#ab570a',
  cyan: '#50e3c2',
  cyanLighter: '#aaffec',
  cyanLight: '#79ffe1',
  cyanDark: '#29bc9b',
  violet: '#7928ca',
  violetLighter: '#e3d7fc',
  violetLight: '#8a63d2',
  violetDark: '#4c2889',
  purple: '#f81ce5',
  alert: '#ff0080',
  magenta: '#eb367f',
  link: '#0070f3'
};

export const darkPrimitiveThemes = {
  background: '#000',
  foreground: '#fff',
  accents_1: '#111',
  accents_2: '#333',
  accents_3: '#444',
  accents_4: '#666',
  accents_5: '#888',
  accents_6: '#999',
  accents_7: '#eaeaea',
  accents_8: '#fafafa',
  selection: '#f81ce5',
  secondary: '#888',
  code: '#79ffe1',
  border: '#333',
  error: '#e00',
  errorLight: '#ff1a1a',
  errorLighter: '#f7d4d6',
  errorDark: '#c50000',
  success: '#0070f3',
  successLight: '#3291ff',
  successLighter: '#d3e5ff',
  successDark: '#0761d1',
  warning: '#f5a623',
  warningLight: '#f7b955',
  warningLighter: '#ffefcf',
  warningDark: '#ab570a',
  cyan: '#50e3c2',
  cyanLighter: '#aaffec',
  cyanLight: '#79ffe1',
  cyanDark: '#29bc9b',
  violet: '#7928ca',
  violetLighter: '#e3d7fc',
  violetLight: '#8a63d2',
  violetDark: '#4c2889',
  purple: '#f81ce5',
  alert: '#ff0080',
  magenta: '#eb367f',
  link: '#3291ff'
};

export const colors = defineVars({
  background: {
    default: lightPrimitiveThemes.background,
    [preferDarkQuery]: darkPrimitiveThemes.background
  },
  foreground: {
    default: lightPrimitiveThemes.foreground,
    [preferDarkQuery]: darkPrimitiveThemes.foreground
  },
  accents_1: {
    default: lightPrimitiveThemes.accents_1,
    [preferDarkQuery]: darkPrimitiveThemes.accents_1
  },
  accents_2: {
    default: lightPrimitiveThemes.accents_2,
    [preferDarkQuery]: darkPrimitiveThemes.accents_2
  },
  accents_3: {
    default: lightPrimitiveThemes.accents_3,
    [preferDarkQuery]: darkPrimitiveThemes.accents_3
  },
  accents_4: {
    default: lightPrimitiveThemes.accents_4,
    [preferDarkQuery]: darkPrimitiveThemes.accents_4
  },
  accents_5: {
    default: lightPrimitiveThemes.accents_5,
    [preferDarkQuery]: darkPrimitiveThemes.accents_5
  },
  accents_6: {
    default: lightPrimitiveThemes.accents_6,
    [preferDarkQuery]: darkPrimitiveThemes.accents_6
  },
  accents_7: {
    default: lightPrimitiveThemes.accents_7,
    [preferDarkQuery]: darkPrimitiveThemes.accents_7
  },
  accents_8: {
    default: lightPrimitiveThemes.accents_8,
    [preferDarkQuery]: darkPrimitiveThemes.accents_8
  },
  selection: {
    default: lightPrimitiveThemes.selection,
    [preferDarkQuery]: darkPrimitiveThemes.selection
  },
  secondary: {
    default: lightPrimitiveThemes.secondary,
    [preferDarkQuery]: darkPrimitiveThemes.secondary
  },
  code: {
    default: lightPrimitiveThemes.code,
    [preferDarkQuery]: darkPrimitiveThemes.code
  },
  border: {
    default: lightPrimitiveThemes.border,
    [preferDarkQuery]: darkPrimitiveThemes.border
  },
  error: {
    default: lightPrimitiveThemes.error,
    [preferDarkQuery]: darkPrimitiveThemes.error
  },
  errorLight: {
    default: lightPrimitiveThemes.errorLight,
    [preferDarkQuery]: darkPrimitiveThemes.errorLight
  },
  errorLighter: {
    default: lightPrimitiveThemes.errorLighter,
    [preferDarkQuery]: darkPrimitiveThemes.errorLighter
  },
  errorDark: {
    default: lightPrimitiveThemes.errorDark,
    [preferDarkQuery]: darkPrimitiveThemes.errorDark
  },
  success: {
    default: lightPrimitiveThemes.success,
    [preferDarkQuery]: darkPrimitiveThemes.success
  },
  successLight: {
    default: lightPrimitiveThemes.successLight,
    [preferDarkQuery]: darkPrimitiveThemes.successLight
  },
  successLighter: {
    default: lightPrimitiveThemes.successLighter,
    [preferDarkQuery]: darkPrimitiveThemes.successLighter
  },
  successDark: {
    default: lightPrimitiveThemes.successDark,
    [preferDarkQuery]: darkPrimitiveThemes.successDark
  },
  warning: {
    default: lightPrimitiveThemes.warning,
    [preferDarkQuery]: darkPrimitiveThemes.warning
  },
  warningLight: {
    default: lightPrimitiveThemes.warningLight,
    [preferDarkQuery]: darkPrimitiveThemes.warningLight
  },
  warningLighter: {
    default: lightPrimitiveThemes.warningLighter,
    [preferDarkQuery]: darkPrimitiveThemes.warningLighter
  },
  warningDark: {
    default: lightPrimitiveThemes.warningDark,
    [preferDarkQuery]: darkPrimitiveThemes.warningDark
  },
  cyan: {
    default: lightPrimitiveThemes.cyan,
    [preferDarkQuery]: darkPrimitiveThemes.cyan
  },
  cyanLighter: {
    default: lightPrimitiveThemes.cyanLighter,
    [preferDarkQuery]: darkPrimitiveThemes.cyanLighter
  },
  cyanLight: {
    default: lightPrimitiveThemes.cyanLight,
    [preferDarkQuery]: darkPrimitiveThemes.cyanLight
  },
  cyanDark: {
    default: lightPrimitiveThemes.cyanDark,
    [preferDarkQuery]: darkPrimitiveThemes.cyanDark
  },
  violet: {
    default: lightPrimitiveThemes.violet,
    [preferDarkQuery]: darkPrimitiveThemes.violet
  },
  violetLighter: {
    default: lightPrimitiveThemes.violetLighter,
    [preferDarkQuery]: darkPrimitiveThemes.violetLighter
  },
  violetLight: {
    default: lightPrimitiveThemes.violetLight,
    [preferDarkQuery]: darkPrimitiveThemes.violetLight
  },
  violetDark: {
    default: lightPrimitiveThemes.violetDark,
    [preferDarkQuery]: darkPrimitiveThemes.violetDark
  },
  purple: {
    default: lightPrimitiveThemes.purple,
    [preferDarkQuery]: darkPrimitiveThemes.purple
  },
  alert: {
    default: lightPrimitiveThemes.alert,
    [preferDarkQuery]: darkPrimitiveThemes.alert
  },
  magenta: {
    default: lightPrimitiveThemes.magenta,
    [preferDarkQuery]: darkPrimitiveThemes.magenta
  },
  link: {
    default: lightPrimitiveThemes.link,
    [preferDarkQuery]: darkPrimitiveThemes.link
  }
});

export const lightTheme = createTheme(colors, lightPrimitiveThemes);

export const darkTheme = createTheme(colors, darkPrimitiveThemes);
