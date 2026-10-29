import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { userMessage } from '../lib/userMessage'

export const useProductsStore = defineStore('products', {
  state: () => ({
    products: [],
    loading: false,
    error: '',
    message: '',
  }),
  actions: {
    async load() {
      this.loading = true
      this.error = ''
      try {
        this.products = await invoke('list_products')
      } catch (cause) {
        this.error = userMessage(cause)
      } finally {
        this.loading = false
      }
    },
    async createProduct(payload) {
      this.error = ''
      try {
        await invoke('create_product', { product: payload })
        this.message = 'Produto cadastrado.'
        await this.load()
        return true
      } catch (cause) {
        this.error = userMessage(cause)
        return false
      }
    },
    async addStock(payload) {
      this.error = ''
      try {
        await invoke('add_stock', { stock: payload })
        this.message = 'Entrada de estoque registrada.'
        await this.load()
        return true
      } catch (cause) {
        this.error = userMessage(cause)
        return false
      }
    },
  },
})
