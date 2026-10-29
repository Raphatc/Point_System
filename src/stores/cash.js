import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { userMessage } from '../lib/userMessage'

export const useCashStore = defineStore('cash', {
  state: () => ({
    status: { is_open: false, opening_amount: 0, expected_cash: 0, pix_total: 0, card_total: 0 },
    openingAmount: 0,
    countedAmount: 0,
    movement: { movementType: 'saida', amount: 0, method: 'dinheiro', description: '' },
    closing: null,
    busy: false,
    error: '',
    message: '',
    loading: true,
  }),
  actions: {
    async refresh() {
      this.loading = true
      this.error = ''
      try {
        this.status = await invoke('get_cash_status')
        this.countedAmount = this.status.expected_cash / 100
      } catch (cause) {
        this.error = userMessage(cause)
      } finally {
        this.loading = false
      }
    },
    async run(action) {
      this.busy = true
      this.error = ''
      this.message = ''
      try {
        await action()
        await this.refresh()
      } catch (cause) {
        this.error = userMessage(cause)
      } finally {
        this.busy = false
      }
    },
    async openCash() {
      await this.run(async () => {
        await invoke('open_cash_session', {
          openingAmount: Math.round(Number(this.openingAmount) * 100),
        })
        this.message = 'Caixa aberto.'
      })
    },
    async saveMovement() {
      await this.run(async () => {
        await invoke('register_cash_movement', {
          movementType: this.movement.movementType,
          amount: Math.round(Number(this.movement.amount) * 100),
          method: this.movement.method,
          description: this.movement.description.trim(),
        })
        this.message = 'Movimento registrado.'
        this.movement = { movementType: 'saida', amount: 0, method: 'dinheiro', description: '' }
      })
    },
    async closeCash() {
      await this.run(async () => {
        this.closing = await invoke('close_cash_session', {
          countedAmount: Math.round(Number(this.countedAmount) * 100),
        })
        this.message = 'Caixa fechado e conferido.'
      })
    },
  },
})
