import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { userMessage } from '../lib/userMessage'

export const useSellersStore = defineStore('sellers', {
  state: () => ({
    sellers: [],
    loading: false,
    error: '',
    message: '',
  }),
  actions: {
    async load() {
      this.loading = true
      this.error = ''
      try {
        this.sellers = await invoke('list_sellers')
      } catch (cause) {
        this.error = userMessage(cause)
      } finally {
        this.loading = false
      }
    },
    async createSeller(payload) {
      this.error = ''
      try {
        await invoke('create_seller', { seller: payload })
        this.message = 'Vendedor cadastrado.'
        await this.load()
        return true
      } catch (cause) {
        this.error = userMessage(cause)
        return false
      }
    },
  },
})
