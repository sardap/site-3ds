import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import App from './App.vue'
import { messages } from './messages'

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: messages,
  datetimeFormats: {
    en: {
      date: {
        day: '2-digit',
        month: '2-digit',
        year: 'numeric',
      },
    },
    kr: {
      date: {
        month: '2-digit',
        day: '2-digit',
        year: 'numeric',
      },
    },
  },
})

const app = createApp(App)

app.use(i18n)
app.use(createPinia())

app.mount('#app')
