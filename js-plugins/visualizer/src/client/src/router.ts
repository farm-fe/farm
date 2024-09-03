import { createRouter, createWebHistory } from 'vue-router';

export const routes = [
  {
    path: '/',
    component: () => import('./pages/Home.vue')
  },
  {
    path: '/dashboard',
    component: () => import('./pages/Home.vue')
  },
  {
    path: '/analysis/compilation',
    component: () => import('./pages/analysis/Compilation.vue')
  },
  {
    path: '/analysis/bundle',
    component: () => import('./pages/analysis/Bundle.vue')
  },
  // {
  //   path: '/analysis/module',
  //   component: () => import('./pages/analysis/Module.vue')
  // },
  {
    path: '/analysis/plugin',
    component: () => import('./pages/analysis/Plugin.vue')
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;
