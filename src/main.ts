import { createApp } from 'vue';
import { createPinia } from 'pinia';
import '@topdata-software-gmbh/design-tokens/tokens.css';
import '@topdata-software-gmbh/ui-kit-web-vue3/styles.css';
import App from './App.vue';
import './style.css';

createApp(App).use(createPinia()).mount('#app');
