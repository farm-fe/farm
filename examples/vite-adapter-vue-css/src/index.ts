import { createApp } from "vue";
import { createRouter, createWebHistory } from "vue-router";
import App from "./app.vue";

import './index.css';

const router = createRouter({
  routes: [
    {
      path: "/",
      redirect: "/box",
      name: "index",
      component: () => import("./views/index.vue"),
    },
    {
      path: "/box",
      name: "box",
      component: () => import("./views/box.vue"),
    },
  ],
  history: createWebHistory("/"),
});
createApp(App).use(router).mount("#app");
