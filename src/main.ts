import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { check, Update } from '@tauri-apps/plugin-updater'
import App from './AppShell.vue'
import './style.css'

async function checkForUpdatesAndInstall(): Promise<void> {
  try {
    const update: Update | null = await check()
    if (update) {
      await update.downloadAndInstall()
    }
  } catch {
    // Atualizações indisponíveis (ex.: sem endpoint configurado). Ignora.
  }
}

checkForUpdatesAndInstall()

createApp(App).use(createPinia()).mount('#app')
