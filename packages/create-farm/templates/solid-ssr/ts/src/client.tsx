import { hydrate } from 'solid-js/web';

import { routes } from './main';
import { Router } from '@solidjs/router';

hydrate(() => <Router>{routes}</Router>, document.getElementById('root'));
