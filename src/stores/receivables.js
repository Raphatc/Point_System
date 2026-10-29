import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { userMessage } from '../lib/userMessage'
import { formatCurrency } from '../lib/format'

export const useReceivablesStore = defineStore('receivables', {
  state: () => ({
    receivables: [],
    payments: {},
    busyId: null,
    error: '',
    message: '',
    loading: true,
  }),
  actions: {
    async load() {
      this.loading = true
      this.error = ''
      try {
        this.receivables = await invoke('list_receivables')
        for (const item of this.receivables) {
          for (const installment of item.installments) {
            const key = installment.id ?? `legacy-${item.sale_id}`
            this.payments[key] ??= { amount: installment.balance / 100, method: 'dinheiro' }
          }
        }
      } catch (cause) {
        this.error = userMessage(cause)
      } finally {
        this.loading = false
      }
    },
    async receive(item, installment) {
      try {
        this.error = ''
        const key = installment.id ?? `legacy-${item.sale_id}`
        this.busyId = key
        const payment = this.payments[key] || {
          amount: installment.balance / 100,
          method: 'dinheiro',
        }
        const balance = await invoke('receive_receivable_payment', {
          saleId: item.sale_id,
          installmentId: installment.id,
          amount: Math.round(Number(payment.amount) * 100),
          method: payment.method,
        })
        this.message =
          balance === 0
            ? `Parcela ${installment.installment_number} da venda #${item.sale_id} quitada.`
            : `Recebimento registrado. Saldo da parcela: ${formatCurrency(balance)}.`
        if (balance === 0) {
          delete this.payments[key]
        } else {
          this.payments[key] = { ...payment, amount: balance / 100 }
        }
        await this.load()
      } catch (cause) {
        this.error = userMessage(cause)
      } finally {
        this.busyId = null
      }
    },
  },
})
