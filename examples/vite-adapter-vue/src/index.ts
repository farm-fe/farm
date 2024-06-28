import { createApp } from 'vue'
import './style.css'
import './style.sass'
import App from './App.vue'

import { createRouter, createWebHistory } from 'vue-router/auto'
import { routes } from 'vue-router/auto/routes';

import SvgIcon from '~virtual/svg-component'

import 'ant-design-vue/dist/antd.less';
import "bootstrap/scss/bootstrap.scss"

const app = createApp(App);

app.component(SvgIcon.name, SvgIcon)
const router = createRouter({
  history: createWebHistory(),
  routes
});

app.use(router).mount('#root');
