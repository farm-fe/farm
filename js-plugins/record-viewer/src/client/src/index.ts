// register vue composition api globally
import { createApp } from 'vue';
import App from './App.vue';

import './style.css';
const app = createApp(App);

// app.use(router)
app.mount('#app');
