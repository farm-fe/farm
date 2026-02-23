import { createWebHistory } from 'vue-router';
import { createFarmSsrVueApp } from './app';
import './styles/app.less';

const HOME_BANNER = 'Farm SSR Toolkit';

const { app, router } = createFarmSsrVueApp({
  history: createWebHistory(),
  homeBanner: HOME_BANNER
});

router.isReady().then(() => {
  app.mount('#root');
});

if (import.meta.hot) {
  import.meta.hot.accept();
}
