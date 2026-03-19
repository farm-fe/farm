import { renderToString } from 'vue/server-renderer';
import { createMemoryHistory } from 'vue-router';
import { createFarmSsrVueApp } from './app';

const HOME_BANNER = 'Farm SSR Toolkit';

export default async function render(url) {
  const { app, router } = createFarmSsrVueApp({
    history: createMemoryHistory(),
    homeBanner: HOME_BANNER
  });

  await router.push(url);
  await router.isReady();

  return renderToString(app);
}
