import { globalStyle, style } from '@vanilla-extract/css';

globalStyle(':root', {
  fontFamily: 'Inter, system-ui, Avenir, Helvetica, Arial, sans-serif',
  lineHeight: '1.5',
  fontWeight: '400',
  colorScheme: 'light dark',
  color: 'rgba(255, 255, 255, 0.87)',
  fontSynthesis: 'none',
  textRendering: 'optimizeLegibility',
  WebkitFontSmoothing: 'antialiased',
  MozOsxFontSmoothing: 'grayscale'
});

globalStyle('a', {
  fontWeight: '500',
  color: '#646cff',
  textDecoration: 'inherit'
});

globalStyle('a:hover', {
  color: '#535bf2'
});

globalStyle('body', {
  margin: '0',
  display: 'flex',
  placeItems: 'center',
  minWidth: '320px',
  minHeight: '100vh',
  backgroundColor: '#242424',
});

globalStyle('h1', {
  fontSize: '3.2em',
  lineHeight: '1.1'
});

globalStyle('button', {
  borderRadius: '8px',
  border: '1px solid transparent',
  padding: '0.6em 1.2em',
  fontSize: '1em',
  fontWeight: '500',
  fontFamily: 'inherit',
  backgroundColor: '#1a1a1a',
  cursor: 'pointer',
  transition: 'border-color 0.25s'
});

globalStyle('button:hover', {
  borderColor: '#646cff'
});

globalStyle('button:focus, button:focus-visible', {
  outline: '4px auto -webkit-focus-ring-color'
});

export const read_the_docs = style({
  color: '#888'
});

export const card = style({
  padding: '2rem'
});

export const logo = style({
  height: '6em',
  padding: '1.5em',
  willChange: 'filter',
  transition: 'filter 300ms',
  selectors: {
    '&:hover': {
      filter: 'drop-shadow(0 0 2em #9F1A8Faa)'
    },
    '&.vanilla:hover': {
      filter: 'drop-shadow(0 0 2em #3178c6aa)'
    }
  }
});
