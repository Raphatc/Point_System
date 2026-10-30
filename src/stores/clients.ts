import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { userMessage } from '../lib/userMessage'
import type { Client } from '../types'

export const useClientsStore = defineStore('clients', {
  state: () => ({
    clients: [] as Client[],
    loading: false,
    error: '',
    message: '',
  }),
  actions: {
    async load(): Promise<void> {
      this.loading = true
      this.error = ''
      try {
        this.clients = await invoke<Client[]>('list_clients')
      } catch (cause) {
        this.error = userMessage(cause)
      } finally {
        this.loading = false
      }
    },
    async createClient(payload: { name: string }): Promise<boolean> {
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
