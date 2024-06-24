import { createSSRApp } from 'vue';

import { createRoute } from './routes';
import { createWebHistory } from 'vue-router';
import Main from './main.vue';
import PrimeVue from "primevue/config";
import Button from 'primevue/button';

const app = createSSRApp(Main);

app.use(PrimeVue, {
  unstyled: true
});
app.component('Button', Button);
const router = createRoute(createWebHistory());
app.use(router);

router.isReady().then(() => {
  app.mount('#root');
});
