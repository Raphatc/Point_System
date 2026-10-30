import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { userMessage } from '../lib/userMessage'
import type { Product } from '../types'

export const useProductsStore = defineStore('products', {
  state: () => ({
    products: [] as Product[],
    loading: false,
    error: '',
    message: '',
  }),
  actions: {
    async load(): Promise<void> {
      this.loading = true
      this.error = ''
      try {
        this.products = await invoke<Product[]>('list_products')
      } catch (cause) {
        this.error = userMessage(cause)
      } finally {
        this.loading = false
      }
    },
    async createProduct(payload: {
      name: string
      sku: string
      gtin: string
      category: string
      unit_price: number
      cost_price: number
    }): Promise<boolean> {
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
    async addStock(payload: {
      product_id: number
      quantity: number
      cost_price: number
      expiration_date: string
      supplier: string
    }): Promise<boolean> {
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
