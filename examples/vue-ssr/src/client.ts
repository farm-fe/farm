import { createSSRApp } from 'vue';

import { createRoute } from './routes';
import { createWebHistory } from 'vue-router';
import Main from './main.vue';

const app = createSSRApp(Main);

const router = createRoute(createWebHistory());
app.use(router);

router.isReady().then(() => {
  app.mount('#root');
});
