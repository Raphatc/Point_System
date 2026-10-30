import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { userMessage } from '../lib/userMessage'
import { formatCurrency } from '../lib/format'
import type { Receivable } from '../types'

export const useReceivablesStore = defineStore('receivables', {
  state: () => ({
    receivables: [] as Receivable[],
    payments: {} as Record<string, { amount: number; method: string }>,
    busyId: null as string | number | null,
    error: '',
    message: '',
    loading: true,
  }),
  actions: {
    async load(): Promise<void> {
      this.loading = true
      this.error = ''
      try {
        this.receivables = await invoke<Receivable[]>('list_receivables')
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
    async receive(
      item: Receivable,
      installment: { id: number | null; installment_number: number; balance: number },
    ): Promise<number> {
      try {
        this.error = ''
        const key = installment.id ?? `legacy-${item.sale_id}`
        this.busyId = key
        const payment = this.payments[key] || {
          amount: installment.balance / 100,
          method: 'dinheiro',
        }
        const balance = await invoke<number>('receive_receivable_payment', {
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
        return balance
      } catch (cause) {
        this.error = userMessage(cause)
        return 0
      } finally {
        this.busyId = null
      }
    },
  },
})
