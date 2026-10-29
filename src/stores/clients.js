import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { userMessage } from '../lib/userMessage'

export const useClientsStore = defineStore('clients', {
  state: () => ({
    clients: [],
    loading: false,
    error: '',
    message: '',
  }),
  actions: {
    async load() {
      this.loading = true
      this.error = ''
      try {
        this.clients = await invoke('list_clients')
      } catch (cause) {
        this.error = userMessage(cause)
      } finally {
        this.loading = false
      }
    },
    async createClient(payload) {
      this.error = ''
      try {
        await invoke('create_client', { client: payload })
        this.message = 'Cliente cadastrado.'
        await this.load()
        return true
      } catch (cause) {
        this.error = userMessage(cause)
        return false
      }
    },
  },
})
