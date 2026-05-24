import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { createRouter, createWebHashHistory } from 'vue-router';

import App from './App.vue';
import HomeView from './views/HomeView.vue';
import AboutView from './views/AboutView.vue';
import './style.css';

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', component: HomeView },
    { path: '/about', component: AboutView },
  ],
});

createApp(App).use(createPinia()).use(router).mount('#root');
