<script setup>
import { onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { formatCurrency } from '../../lib/format'
import { useCashStore } from '../../stores/cash'
import CashIdleScreen from './CashIdleScreen.vue'
import OpenCashDialog from './OpenCashDialog.vue'
import CloseCashDialog from './CloseCashDialog.vue'

const cashStore = useCashStore()
const { status, openingAmount, countedAmount, movement, closing, error, message, busy, loading } =
  storeToRefs(cashStore)
const openCashOpen = ref(false)
const quickCloseOpen = ref(false)

async function refresh() {
  await cashStore.refresh()
}

function openCash() {
  return cashStore.openCash().then(() => {
    openCashOpen.value = false
  })
}

function saveMovement() {
  return cashStore.saveMovement()
}

function closeCash() {
  return cashStore.closeCash().then(() => {
    quickCloseOpen.value = false
  })
}

function confirmQuickClose(amount) {
  countedAmount.value = amount
  closeCash()
}

function openQuickClose() {
  quickCloseOpen.value = true
}

onMounted(refresh)
</script>

<template>
  <section class="narrow-content">
    <div v-if="loading" class="loading-state" role="status">Consultando caixa...</div>
    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <div v-if="message" class="notice success">{{ message }}</div>

    <CashIdleScreen v-if="!loading && !status.is_open" @open="openCashOpen = true" :busy="busy" />

    <article v-else class="card">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Controle diário</p>
          <h3>Caixa aberto</h3>
        </div>
        <span class="status-pill open">Em operação</span>
      </div>
      <div class="cash-stats">
        <div>
          <small>Dinheiro esperado</small
          ><strong>{{ formatCurrency(status.expected_cash) }}</strong>
        </div>
        <div>
          <small>PIX no turno</small><strong>{{ formatCurrency(status.pix_total) }}</strong>
        </div>
        <div>
          <small>Cartão no turno</small><strong>{{ formatCurrency(status.card_total) }}</strong>
        </div>
      </div>
      <form class="cash-movement" @submit.prevent="saveMovement">
        <h4>Lançamento manual</h4>
        <div class="field-grid compact">
          <select v-model="movement.movementType">
            <option value="entrada">Entrada</option>
            <option value="saida">Saída</option>
          </select>
          <input
            v-model.number="movement.amount"
            type="number"
            min="0.01"
            step="0.01"
            required
            placeholder="Valor"
          />
          <select v-model="movement.method">
            <option value="dinheiro">Dinheiro</option>
            <option value="pix">PIX</option>
            <option value="cartao_credito">Cartão de crédito</option>
            <option value="cartao_debito">Cartão de débito</option>
          </select>
          <input v-model="movement.description" required placeholder="Descrição" />
        </div>
        <button class="secondary" type="submit" :disabled="busy">Registrar movimento</button>
      </form>
      <form class="cash-closing" @submit.prevent="closeCash">
        <h4>Fechamento e conferência</h4>
        <label
          >Dinheiro contado<input
            v-model.number="countedAmount"
            type="number"
            min="0"
            step="0.01"
            required
        /></label>
        <button class="primary" type="button" :disabled="busy" @click="openQuickClose">
          Fechar caixa
        </button>
      </form>
      <p v-if="closing" class="notice" :class="closing.difference === 0 ? 'success' : 'warning'">
        Esperado {{ formatCurrency(closing.expected_amount) }}; contado
        {{ formatCurrency(closing.counted_amount) }}; diferença
        {{ formatCurrency(closing.difference) }}.
      </p>
    </article>

    <OpenCashDialog
      :open="openCashOpen"
      :openingAmount="openingAmount"
      :busy="busy"
      @open="openCash"
      @cancel="openCashOpen = false"
    />
    <CloseCashDialog
      :open="quickCloseOpen"
      :expectedCash="status.expected_cash"
      @close="confirmQuickClose"
      @cancel="quickCloseOpen = false"
    />
  </section>
</template>
