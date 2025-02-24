import { useApplicationContext, useToggleTheme } from '../../context';
import type { Theme } from '../../context';
import { colors } from '../../themes/color.stylex';
import * as stylex from '@stylexjs/stylex';
import { Input } from '../input';
import { Select } from '../select';
import type { SelectOptionProps } from '../select/select-option';
import { Button } from '../button';
import { Spacer } from '../spacer';

const Icons = {
  Moon: () => (
    <svg
      id='theme-light'
      xmlns='http://www.w3.org/2000/svg'
      width='18'
      height='18'
      viewBox='0 0 256 256'
    >
      <path
        fill='currentColor'
        d='M233.54 142.23a8 8 0 0 0-8-2a88.08 88.08 0 0
      1-109.8-109.8a8 8 0 0 0-10-10a104.84 104.84 0 0 0-52.91 37A104 104 0 0 0
      136 224a103.1 103.1 0 0 0 62.52-20.88a104.84 104.84 0 0 0 37-52.91a8 8 0
      0 0-1.98-7.98m-44.64 48.11A88 88 0 0 1 65.66 67.11a89 89 0 0 1
      31.4-26A106 106 0 0 0 96 56a104.11 104.11 0 0 0 104 104a106 106 0 0 0
      14.92-1.06a89 89 0 0 1-26.02 31.4'
      />
    </svg>
  ),
  Sun: () => (
    <svg
      id='theme-dark'
      xmlns='http://www.w3.org/2000/svg'
      width='18'
      height='18'
      viewBox='0 0 256 256'
    >
      <path
        fill='currentColor'
        d='M120 40V16a8 8 0 0 1 16 0v24a8 8 0 0 1-16 0m72
      88a64 64 0 1 1-64-64a64.07 64.07 0 0 1 64 64m-16 0a48 48 0 1 0-48 48a48.05
      48.05 0 0 0 48-48M58.34 69.66a8 8 0 0 0 11.32-11.32l-16-16a8 8 0 0
      0-11.32 11.32Zm0 116.68l-16 16a8 8 0 0 0 11.32 11.32l16-16a8 8 0 0
      0-11.32-11.32M192 72a8 8 0 0 0 5.66-2.34l16-16a8 8 0 0 0-11.32-11.32l-16
      16A8 8 0 0 0 192 72m5.66 114.34a8 8 0 0 0-11.32 11.32l16 16a8 8 0 0 0
      11.32-11.32ZM48 128a8 8 0 0 0-8-8H16a8 8 0 0 0 0 16h24a8 8 0 0 0
      8-8m80 80a8 8 0 0 0-8 8v24a8 8 0 0 0 16 0v-24a8 8 0 0 0-8-8m112-88h-24a8
      8 0 0 0 0 16h24a8 8 0 0 0 0-16'
      />
    </svg>
  ),
  GitHub: () => (
    <svg
      xmlns='http://www.w3.org/2000/svg'
      width='32'
      height='32'
      viewBox='0 0 256 256'
    >
      <path
        fill='currentColor'
        d='M208.31 75.68A59.78 59.78 0 0 0 202.93 28a8 8
      0 0 0-6.93-4a59.75 59.75 0 0 0-48 24h-24a59.75 59.75 0 0 0-48-24a8 8 0 0
      0-6.93 4a59.78 59.78 0 0 0-5.38 47.68A58.14 58.14 0 0 0 56 104v8a56.06
      56.06 0 0 0 48.44 55.47A39.8 39.8 0 0 0 96 192v8H72a24 24 0 0 1-24-24a40
      40 0 0 0-40-40a8 8 0 0 0 0 16a24 24 0 0 1 24 24a40 40 0 0 0 40 40h24v16a8
      8 0 0 0 16 0v-40a24 24 0 0 1 48 0v40a8 8 0 0 0 16 0v-40a39.8 39.8 0 0
      0-8.44-24.53A56.06 56.06 0 0 0 216 112v-8a58.14 58.14 0 0 0-7.69-28.32M200
      112a40 40 0 0 1-40 40h-48a40 40 0 0 1-40-40v-8a41.74 41.74 0 0 1
      6.9-22.48a8 8 0 0 0 1.1-7.69a43.8 43.8 0 0 1 .79-33.58a43.88 43.88 0 0 1
      32.32 20.06a8 8 0 0 0 6.71 3.69h32.35a8 8 0 0 0 6.74-3.69a43.87 43.87 0 0
      1 32.32-20.06a43.8 43.8 0 0 1 .77 33.58a8.09 8.09 0 0 0 1 7.65a41.7 41.7
      0 0 1 7 22.52Z'
      />
    </svg>
  ),
  Menu: () => (
    <svg
      xmlns='http://www.w3.org/2000/svg'
      width='32'
      height='32'
      viewBox='0 0 256 256'
    >
      <path
        fill='#888888'
        d='M224 128a8 8 0 0 1-8 8H40a8 8 0 0 1 0-16h176a8 8 0 0 1 8 8M40 
  72h176a8 8 0 0 0 0-16H40a8 8 0 0 0 0 16m176 112H40a8 8 0 0 0 0 16h176a8 8 0 0 0 0-16'
      />
    </svg>
  )
};

const styles = stylex.create({
  selectOptions: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    width: 'auto',
    height: '18px',
    ':not(#_) svg': {
      marginRight: '10px',
      marginLeft: '2px'
    }
  }
});

const THEME_OPTIONS: SelectOptionProps[] = [
  {
    value: 'light',
    label: (
      <span {...stylex.props(styles.selectOptions)}>
        <Icons.Sun />
        Light
      </span>
    )
  },
  {
    value: 'dark',
    label: (
      <span {...stylex.props(styles.selectOptions)}>
        <Icons.Moon stylex={{ height: '14px' }} />
        Dark
      </span>
    )
  }
];

const CONTROL_OPTIONS: SelectOptionProps[] = [
  { value: 'inspect', label: 'Plugin Inspect' },
  { value: 'analysis', label: 'Module Analysis' }
];

export function NavHead() {
  const { theme } = useApplicationContext();

  const toggleTheme = useToggleTheme();

  const handleGitHubButtonClick = () => {
    window.open('https://www.farmfe.org/');
  };

  return (
    <nav
      stylex={{
        height: '64px',
        boxSizing: 'border-box',
        background: colors.background,
        padding: '12px',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center'
      }}
    >
      <div id='search-container'>
        <Input placeholder='Search Resource...' clearable />
      </div>
      <div id='nav-widget' stylex={{ display: 'flex', alignItems: 'center' }}>
        <Select
          h='28px'
          w='120px'
          scale={0.5}
          pure
          value={'inspect'}
          options={CONTROL_OPTIONS}
        />
        <Spacer w={0.75} inline />
        <Select
          h='28px'
          w='85px'
          scale={0.5}
          pure
          onChange={toggleTheme}
          value={theme}
          options={THEME_OPTIONS}
        />
        <Spacer w={0.75} inline />
        <Button
          w='28px'
          h='28px'
          py={0}
          px={0}
          icon={<Icons.GitHub />}
          onClick={handleGitHubButtonClick}
        />
      </div>
    </nav>
  );
}
