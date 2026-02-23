import { createSSRApp } from 'vue';
import type { RouterHistory } from 'vue-router';
import AppRoot from './App.vue';
import { createAppRouter } from './router';

export interface CreateFarmSsrVueAppOptions {
  history: RouterHistory;
  homeBanner: string;
}

export function createFarmSsrVueApp(options: CreateFarmSsrVueAppOptions) {
  const app = createSSRApp(AppRoot);
  const router = createAppRouter(options.history);

  app.provide('homeBanner', options.homeBanner);
  app.use(router);

  return {
    app,
    router
  };
}
