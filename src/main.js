import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './AppShell.vue'
import './style.css'

createApp(App).use(createPinia()).mount('#app')
