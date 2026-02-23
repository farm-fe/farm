import { createRouter, type RouteRecordRaw, type RouterHistory } from 'vue-router';
import HomePage from './pages/HomePage.vue';
import NotFoundPage from './pages/NotFoundPage.vue';
import ProductsPage from './pages/ProductsPage.vue';
import { notFoundRoute, routes } from '../smoke/routes.mjs';

const routeRecords: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: HomePage,
    meta: {
      key: 'home',
      heading: routes[0].heading,
      status: 'ok'
    }
  },
  {
    path: '/about',
    name: 'about',
    component: () => import('./pages/AboutPage.vue'),
    meta: {
      key: 'about',
      heading: routes[1].heading,
      status: 'ok'
    }
  },
  {
    path: '/products',
    name: 'products',
    component: ProductsPage,
    meta: {
      key: 'products',
      heading: routes[2].heading,
      status: 'ok'
    }
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'not-found',
    component: NotFoundPage,
    meta: {
      key: 'not-found',
      heading: notFoundRoute.heading,
      status: 'not-found'
    }
  }
];

export function createAppRouter(history: RouterHistory) {
  return createRouter({
    history,
    routes: routeRecords
  });
}
