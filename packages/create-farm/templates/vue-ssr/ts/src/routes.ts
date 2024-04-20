import { RouteRecordRaw, createRouter, RouterHistory } from 'vue-router';
const routes = [
  {
    path: '/',
    name: 'layout',
    component: () => import('./pages/Layout.vue'),
    children: [
      {
        path: '',
        name: 'home',
        component: () => import('./pages/Home.vue')
      },
      {
        path: 'about',
        name: 'about',
        component: () => import('./pages/About.vue')
      },
      {
        path: 'dashboard',
        name: 'dashboard',
        component: () => import('./pages/Dashboard.vue')
      },
      {
        path: '/:pathMatch(.*)*',
        name: '404',
        component: () => import('./pages/NoMatch.vue')
      }
    ]
  }
] satisfies RouteRecordRaw[];

export const createRoute = (history: RouterHistory) =>
  createRouter({
    history,
    routes
  });
