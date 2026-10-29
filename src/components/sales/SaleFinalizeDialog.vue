<script setup>
import { computed, ref } from 'vue'
import { formatCurrency } from '../../lib/format'

const props = defineProps({
  open: { type: Boolean, default: false },
  method: { type: String, default: 'dinheiro' },
  payments: { type: Array, default: () => [] },
  discount: { type: Number, default: 0 },
  surcharge: { type: Number, default: 0 },
  netCents: { type: Number, default: 0 },
  paidCents: { type: Number, default: 0 },
  changeCents: { type: Number, default: 0 },
  remainingCents: { type: Number, default: 0 },
  disabled: { type: Boolean, default: false },
  busy: { type: Boolean, default: false },
  isCredit: { type: Boolean, default: false },
  installments: { type: Array, default: () => [] },
  clients: { type: Array, default: () => [] },
  clientId: { type: [String, Number], default: '' },
  dueDate: { type: String, default: '' },
  installmentsCount: { type: Number, default: 1 },
})

const emit = defineEmits([
  'submit',
  'cancel',
  'update:method',
  'update:discount',
  'update:surcharge',
  'add-payment',
  'remove-payment',
  'update:clientId',
  'update:dueDate',
  'update:installmentsCount',
  'update:payment',
  'update:payment-method',
])

const clientSearch = ref('')

const filteredClients = computed(() => {
  const term = clientSearch.value.trim().toLocaleLowerCase('pt-BR')
  if (!term) return []
  return props.clients.filter((client) => client.name.toLocaleLowerCase('pt-BR').includes(term))
})

function handleRemovePayment() {
  emit('remove-payment', props.payments.length - 1)
}
</script>

<template>
  <div v-if="open" class="dav-modal-backdrop">
    <div class="dav-window" role="dialog" aria-modal="true" aria-label="Finalizar venda">
      <div class="dav-window-header">
        <span>Finalizar a Venda</span>
        <button
          class="dav-window-close"
          type="button"
          aria-label="Fechar diálogo"
          @click="emit('cancel')"
        >
          ×
        </button>
      </div>
      <div class="dav-sub-header">Informações do Pagamento</div>
      <form @submit.prevent="emit('submit')">
        <div class="dav-window-body">
          <div class="finalize-total">
            <span>Total da venda</span>
            <strong>{{ formatCurrency(netCents) }}</strong>
          </div>

          <div v-if="isCredit" class="credit-checkout">
            <h4>Pesquisa de Cliente</h4>
            <label for="final-client-search">Buscar pelo nome</label>
            <input
              id="final-client-search"
              v-model="clientSearch"
              type="search"
              autocomplete="off"
              placeholder="Digite o nome do cliente"
            />
            <ul v-if="filteredClients.length" class="product-results client-results">
              <li v-for="client in filteredClients" :key="client.id">
                <strong>{{ client.name }}</strong>
                <button
                  class="link-button"
                  type="button"
                  @click="emit('update:clientId', client.id)"
                >
                  Selecionar
                </button>
              </li>
            </ul>
            <div class="field-grid compact installment-options">
              <label
                >Primeiro vencimento<input
                  :value="dueDate"
                  type="date"
                  :min="new Date().toLocaleDateString('sv-SE')"
                  required
                  @change="(event) => emit('update:dueDate', event.target.value)"
              /></label>
              <label
                >Número de parcelas<input
                  :value="installmentsCount"
                  type="number"
                  min="1"
                  max="36"
                  required
                  @change="(event) => emit('update:installmentsCount', event.target.value)"
              /></label>
              <label
                >Valor a parcelar<strong class="installment-total">{{
                  formatCurrency(netCents)
                }}</strong></label
              >
            </div>
            <table v-if="installments.length" class="installment-table">
              <thead>
                <tr>
                  <th>Parcela</th>
                  <th>Valor</th>
                  <th>Vencimento</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="installment in installments" :key="installment.number">
                  <td>{{ installment.number }} / {{ installments.length }}</td>
                  <td>{{ formatCurrency(installment.amount) }}</td>
                  <td>
                    {{ new Date(`${installment.dueDate}T00:00:00`).toLocaleDateString('pt-BR') }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <div v-else>
            <div class="finalize-payment-top">
              <label
                >Forma de pagamento
                <select
                  :value="method"
                  @change="(event) => emit('update:method', event.target.value)"
                >
                  <option value="dinheiro">DINHEIRO</option>
                  <option value="cartao_credito">CARTÃO DE CRÉDITO</option>
                  <option value="cartao_debito">CARTÃO DE DÉBITO</option>
                  <option value="pix">PIX</option>
                </select>
              </label>
              <label
                >Valor total
                <input :value="formatCurrency(netCents)" disabled />
              </label>
              <button
                class="secondary"
                type="button"
                :disabled="payments.length >= 3"
                @click="emit('add-payment')"
              >
                Adicionar pagamento
              </button>
              <button
                class="danger"
                type="button"
                :disabled="payments.length <= 1"
                @click="handleRemovePayment"
              >
                Remover
              </button>
            </div>
            <div class="payment-grid-wrap">
              <table class="payment-grid">
                <thead>
                  <tr>
                    <th>Descrição</th>
                    <th>Valor</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(payment, index) in payments" :key="index">
                    <td>
                      <select
                        :value="payment.method"
                        @change="
                          (event) => emit('update:payment-method', index, event.target.value)
                        "
                      >
                        <option value="dinheiro">DINHEIRO</option>
                        <option value="cartao_credito">CARTÃO DE CRÉDITO</option>
                        <option value="cartao_debito">CARTÃO DE DÉBITO</option>
                        <option value="pix">PIX</option>
                      </select>
                    </td>
                    <td>
                      <input
                        :value="payment.amount"
                        type="number"
                        min="0"
                        step="0.01"
                        @input="
                          (event) => emit('update:payment', index, Number(event.target.value))
                        "
                      />
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div class="finalize-payment-top">
              <label
                >Desconto
                <input
                  :value="discount"
                  type="number"
                  min="0"
                  step="0.01"
                  @input="(event) => emit('update:discount', Number(event.target.value))"
                />
              </label>
              <label
                >Acréscimo
                <input
                  :value="surcharge"
                  type="number"
                  min="0"
                  step="0.01"
                  @input="(event) => emit('update:surcharge', Number(event.target.value))"
                />
              </label>
            </div>
          </div>

          <div class="payment-summary">
            <div>
              <span>Desconto</span><strong>{{ formatCurrency(Math.round(discount * 100)) }}</strong>
            </div>
            <div>
              <span>Acréscimo</span
              ><strong>{{ formatCurrency(Math.round(surcharge * 100)) }}</strong>
            </div>
            <div>
              <span>Restante</span><strong>{{ formatCurrency(remainingCents) }}</strong>
            </div>
            <div>
              <span>Troco</span><strong class="change">{{ formatCurrency(changeCents) }}</strong>
            </div>
            <div class="highlight">
              <span>Valor recebido</span><strong>{{ formatCurrency(paidCents) }}</strong>
            </div>
          </div>
        </div>
        <div class="dav-window-footer">
          <div class="dav-shortcut-bar">
            <span><kbd>F1</kbd> Cliente</span>
            <span><kbd>F8</kbd> Desconto/Acréscimo</span>
            <span><kbd>F11</kbd> Menu</span>
          </div>
          <div class="dav-actions">
            <button class="secondary" type="button" @click="emit('cancel')">Cancelar</button>
            <button class="primary" type="submit" :disabled="disabled">
              {{ busy ? 'Finalizando...' : 'Finalizar' }}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
</template>
