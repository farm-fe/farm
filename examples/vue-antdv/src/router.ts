import { createRouter, createWebHistory } from 'vue-router';

export const routes = [
  {
    path: '/',
    component: { template: '<div>111</div>' }
  },
  {
    path: '/test',
    component: () => import('/@/test.vue')
  },
  {
    path: '/test1',
    component: () => import('/@/test1.vue'),
    children: [
      {
        path: 'test2',
        component: () => import('/@/test2.vue')
      }
    ]
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes // `routes: routes` 的缩写
});

export default router;
