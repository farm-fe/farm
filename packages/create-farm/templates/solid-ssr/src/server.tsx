import { renderToString } from 'solid-js/web';

import { routes } from './main';
import { Router } from '@solidjs/router';

export default function render(url: string) {
  return renderToString(() => <Router url={url}>{routes}</Router>);
}
