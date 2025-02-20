import { createTheme, defineVars } from '@stylexjs/stylex';

const preferDarkQuery = '@media (prefers-color-scheme: dark)';

export const lightPrimitiveThemes = {
  background: '#fff',
  foreground: '#000'
};

export const darkPrimitiveThemes = {
  background: '#000',
  foreground: '#fff'
};

export const colors = defineVars({
  background: {
    default: lightPrimitiveThemes.background,
    [preferDarkQuery]: darkPrimitiveThemes.background
  },
  foreground: {
    default: lightPrimitiveThemes.foreground,
    [preferDarkQuery]: darkPrimitiveThemes.foreground
  }
});

export const lightTheme = createTheme(colors, lightPrimitiveThemes);

export const darkTheme = createTheme(colors, darkPrimitiveThemes);
