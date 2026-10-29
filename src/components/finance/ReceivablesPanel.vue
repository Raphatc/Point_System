<script setup>
import { onMounted } from 'vue'
import { useReceivablesStore } from '../../stores/receivables'
import { formatCurrency } from '../../lib/format'

const receivablesStore = useReceivablesStore()
const receivables = receivablesStore.receivables
const payments = receivablesStore.payments
const error = receivablesStore.error
const message = receivablesStore.message
const busyId = receivablesStore.busyId
const loading = receivablesStore.loading

function dueStatus(date) {
  if (!date) return { label: 'Sem vencimento', className: 'neutral' }
  const due = new Date(`${date}T00:00:00`)
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const days = Math.round((due - today) / 86400000)
  if (days < 0) return { label: `Vencida há ${Math.abs(days)} dia(s)`, className: 'overdue' }
  if (days <= 7)
    return { label: days === 0 ? 'Vence hoje' : `Vence em ${days} dia(s)`, className: 'due-soon' }
  return { label: 'No prazo', className: 'neutral' }
}

async function load() {
  await receivablesStore.load()
}

async function receive(item, installment) {
  await receivablesStore.receive(item, installment)
}

onMounted(load)
</script>

<template>
  <section>
    <div v-if="loading" class="loading-state" role="status">Carregando contas...</div>
    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <div v-if="message" class="notice success">{{ message }}</div>
    <article class="card table-card">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Crédito</p>
          <h3>Contas a receber</h3>
        </div>
        <span>{{ receivables.length }} em aberto</span>
      </div>
      <div v-if="receivables.length" class="receivable-list">
        <article v-for="item in receivables" :key="item.sale_id" class="receivable-card">
          <header class="receivable-heading">
            <div>
              <strong>Venda #{{ item.sale_id }}</strong
              ><span>{{ item.client_name }}</span>
            </div>
            <div>
              <small>Total {{ formatCurrency(item.total_amount) }}</small
              ><strong>Saldo {{ formatCurrency(item.balance) }}</strong>
            </div>
          </header>
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Parcela</th>
                  <th>Vencimento</th>
                  <th>Valor</th>
                  <th>Recebido</th>
                  <th>Saldo</th>
                  <th>Receber agora</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="installment in item.installments"
                  :key="installment.id ?? `legacy-${item.sale_id}`"
                >
                  <td>{{ installment.installment_number }} / {{ item.installments.length }}</td>
                  <td>
                    <span class="due-badge" :class="dueStatus(installment.due_date).className">{{
                      dueStatus(installment.due_date).label
                    }}</span>
                  </td>
                  <td>{{ formatCurrency(installment.amount) }}</td>
                  <td>{{ formatCurrency(installment.received_amount) }}</td>
                  <td>
                    <strong>{{ formatCurrency(installment.balance) }}</strong>
                  </td>
                  <td>
                    <div v-if="installment.balance > 0" class="receive-controls">
                      <input
                        v-model.number="payments[installment.id ?? `legacy-${item.sale_id}`].amount"
                        type="number"
                        min="0.01"
                        :max="installment.balance / 100"
                        step="0.01"
                        aria-label="Valor a receber"
                      />
                      <select
                        v-model="payments[installment.id ?? `legacy-${item.sale_id}`].method"
                        aria-label="Forma de recebimento"
                      >
                        <option value="dinheiro">Dinheiro</option>
                        <option value="pix">PIX</option>
                        <option value="cartao_credito">Cartão de crédito</option>
                        <option value="cartao_debito">Cartão de débito</option>
                      </select>
                      <button
                        class="primary compact-button"
                        :disabled="busyId === (installment.id ?? `legacy-${item.sale_id}`)"
                        @click="receive(item, installment)"
                      >
                        Receber
                      </button>
                    </div>
                    <span v-else class="status-pill open">Quitada</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>
      </div>
      <p v-else class="empty-state">
        Nenhuma conta pendente. Pagamentos são registrados com caixa aberto.
      </p>
    </article>
  </section>
</template>
