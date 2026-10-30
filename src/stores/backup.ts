import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { userMessage } from '../lib/userMessage'

export const useBackupStore = defineStore('backup', {
  state: () => ({
    backups: [] as string[],
    busy: false,
    error: '',
    message: '',
    loading: false,
  }),
  actions: {
    async load(): Promise<void> {
      this.loading = true
      this.error = ''
      try {
        this.backups = await invoke<string[]>('list_backups')
      } catch (cause) {
        this.error = userMessage(cause)
      } finally {
        this.loading = false
      }
    },
    async create(): Promise<boolean> {
      this.busy = true
      this.error = ''
      this.message = ''
      try {
        const fileName = await invoke<string>('create_backup')
        this.message = `Backup criado com sucesso: ${fileName}`
        await this.load()
        return true
      } catch (cause) {
        this.error = userMessage(cause)
        return false
      } finally {
        this.busy = false
      }
    },
    async restore(fileName: string): Promise<boolean> {
      this.busy = true
      this.error = ''
      this.message = ''
      try {
        await invoke('restore_backup', { fileName })
        this.message = 'Backup restaurado com sucesso.'
        await this.load()
        return true
      } catch (cause) {
        this.error = userMessage(cause)
        return false
      } finally {
        this.busy = false
      }
    },
  },
})
