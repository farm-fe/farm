import Vue from 'vue';
import TDesign from 'tdesign-vue';
import 'tdesign-vue/es/style/index.css';

import App from './App.vue';

Vue.use(TDesign);
new Vue({
  render: (h) => h(App)
}).$mount('#app');