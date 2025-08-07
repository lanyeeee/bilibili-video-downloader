import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import 'virtual:uno.css'
import 'lazysizes'
import 'lazysizes/plugins/parent-fit/ls.parent-fit'

const pinia = createPinia()
const app = createApp(App)

app.use(pinia).mount('#app')
