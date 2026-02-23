import { message } from './ssr-message.mjs';

export const routes = [
  {
    key: 'home',
    path: '/',
    title: 'Farm SSR Toolkit',
    heading: 'Home',
    content: `${message} | home route`
  },
  {
    key: 'about',
    path: '/about',
    title: 'About - Farm SSR Toolkit',
    heading: 'About Farm SSR Toolkit',
    content: 'about-page-from-ssr-toolkit'
  },
  {
    key: 'products',
    path: '/products',
    title: 'Products - Farm SSR Toolkit',
    heading: 'Products',
    content: 'products-page-from-ssr-toolkit'
  }
];

export const notFoundRoute = {
  key: 'not-found',
  title: 'Route Not Found - Farm SSR Toolkit',
  heading: 'Route Not Found'
};

export function getRouteByKey(key) {
  return routes.find((item) => item.key === key);
}

export function getRouteByPath(path) {
  return routes.find((item) => item.path === path);
}
