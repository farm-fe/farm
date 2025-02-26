import { defineVars } from '@stylexjs/stylex';
import { colors } from './color.stylex';

const preferDarkQuery = '@media (prefers-color-scheme: dark)';

export const expressiveness = defineVars({
  linkStyle: {
    default: 'none',
    [preferDarkQuery]: 'none'
  },
  linkHoverStyle: {
    default: 'none',
    [preferDarkQuery]: 'none'
  },
  dropdownBoxShadow: {
    default: '0 4px 4px 0 rgba(0, 0, 0, 0.02)',
    [preferDarkQuery]: '0 0 0 1px #333'
  },
  scrollerStart: {
    default: 'rgba(255, 255, 255, 1)',
    [preferDarkQuery]: 'rgba(255, 255, 255, 1)'
  },
  scrollerEnd: {
    default: 'rgba(255, 255, 255, 0)',
    [preferDarkQuery]: 'rgba(255, 255, 255, 0)'
  },
  shadowSmall: {
    default: '0 5px 10px rgba(0, 0, 0, 0.12)',
    [preferDarkQuery]: '0 0 0 1px #333'
  },
  shadowMedium: {
    default: '0 8px 30px rgba(0, 0, 0, 0.12)',
    [preferDarkQuery]: '0 0 0 1px #333'
  },
  shadowLarge: {
    default: '0 30px 60px rgba(0, 0, 0, 0.12)',
    [preferDarkQuery]: '0 0 0 1px #333'
  },
  portalOpacity: {
    default: '0.25',
    [preferDarkQuery]: '0.75'
  },
  boxHoverStyle: {
    default: colors.accents_1,
    [preferDarkQuery]: colors.accents_1
  }
});
