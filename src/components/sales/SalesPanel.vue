<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { userMessage } from '../../lib/userMessage'
import { useClientsStore } from '../../stores/clients'
import { useProductsStore } from '../../stores/products'
import { useSellersStore } from '../../stores/sellers'
import SaleFinalizeDialog from './SaleFinalizeDialog.vue'
import SaleReceipt from './SaleReceipt.vue'

const productsStore = useProductsStore()
const clientsStore = useClientsStore()
const sellersStore = useSellersStore()
const products = computed(() => productsStore.products)
const clients = computed(() => clientsStore.clients)
const sellers = computed(() => sellersStore.sellers)
const emit = defineEmits(['navigate'])
const searchInput = ref(null)
const productSearch = ref('')
const clientSearch = ref('')
const receiptData = ref(null)
const receiptConfirmOpen = ref(false)
const busy = ref(false)
const error = ref('')
const message = ref('')
const sale = ref({
  sellerId: '',
  clientId: '',
  dueDate: '',
  installmentsCount: 1,
  items: [{ productId: '', quantity: 1, unitPrice: 0 }],
})
const finalizeOpen = ref(false)
const finalMethod = ref('dinheiro')
const finalPayments = ref([{ method: 'dinheiro', amount: 0 }])
const finalDiscount = ref(0)
const finalSurcharge = ref(0)
const finalNetCents = computed(
  () =>
    sale.value.items.reduce(
      (sum, item) =>
        sum + Number(item.quantity || 0) * Math.round(Number(item.unitPrice || 0) * 100),
      0,
    ) -
    Math.round(finalDiscount.value * 100) +
    Math.round(finalSurcharge.value * 100),
)
const finalPaidCents = computed(() =>
  finalPayments.value.reduce(
    (sum, payment) => sum + Math.round(Number(payment.amount || 0) * 100),
    0,
  ),
)
const finalChangeCents = computed(() => Math.max(0, finalPaidCents.value - finalNetCents.value))
const finalRemainingCents = computed(() => finalNetCents.value - finalPaidCents.value)
const finalIsCredit = computed(() => finalMethod.value === 'prazo')
const installmentSchedule = computed(() => {
  if (!finalIsCredit.value || !sale.value.dueDate || sale.value.installmentsCount < 1) return []
  const count = Number(sale.value.installmentsCount)
  const baseAmount = Math.floor(finalNetCents.value / count)
  const remainder = finalNetCents.value % count
  const [year, month, day] = sale.value.dueDate.split('-').map(Number)
  return Array.from({ length: count }, (_, index) => {
    const monthIndex = month - 1 + index
    const targetYear = year + Math.floor(monthIndex / 12)
    const targetMonth = monthIndex % 12
    const lastDay = new Date(targetYear, targetMonth + 1, 0).getDate()
    const dueDate = new Date(targetYear, targetMonth, Math.min(day, lastDay))
    return {
      number: index + 1,
      amount: baseAmount + Number(index < remainder),
      dueDate: dueDate.toLocaleDateString('sv-SE'),
    }
  })
})
const filteredClients = computed(() => {
  const term = clientSearch.value.trim().toLocaleLowerCase('pt-BR')
  if (!term) return []
  return clients.value.filter((client) => client.name.toLocaleLowerCase('pt-BR').includes(term))
})
const selectedClient = computed(() =>
  clients.value.find((client) => client.id === Number(sale.value.clientId)),
)
const filteredProducts = computed(() => {
  const term = productSearch.value.trim().toLocaleLowerCase('pt-BR')
  if (!term) return []
  return products.value.filter(
    (product) =>
      product.name.toLocaleLowerCase('pt-BR').includes(term) ||
      product.sku.toLocaleLowerCase('pt-BR').includes(term) ||
      product.gtin?.toLocaleLowerCase('pt-BR').includes(term),
  )
})

function selectedProduct(id) {
  return products.value.find((item) => item.id === Number(id))
}

function addProduct(product) {
  error.value = ''
  if (product.stock_quantity < 1) {
    error.value = `${product.name} está sem estoque. Registre uma entrada antes de vender.`
    return
  }

  const existing = sale.value.items.find((item) => Number(item.productId) === product.id)
  if (existing) {
    if (existing.quantity >= product.stock_quantity) {
      error.value = `Estoque disponível para ${product.name}: ${product.stock_quantity} unidade(s).`
      return
    }
    existing.quantity += 1
  } else {
    const emptyLine = sale.value.items.find((item) => !item.productId)
    const line = { productId: product.id, quantity: 1, unitPrice: product.unit_price / 100 }
    if (emptyLine) Object.assign(emptyLine, line)
    else sale.value.items.push(line)
  }

  productSearch.value = ''
  nextTick(() => searchInput.value?.focus())
}

function addFromSearch() {
  const query = productSearch.value.trim().toLocaleLowerCase('pt-BR')
  if (!query) return
  const exactSku = products.value.find(
    (product) => product.sku.toLocaleLowerCase('pt-BR') === query,
  )
  if (exactSku) {
    addProduct(exactSku)
    return
  }
  if (filteredProducts.value.length === 1) {
    addProduct(filteredProducts.value[0])
    return
  }
  error.value = filteredProducts.value.length
    ? 'Há mais de um resultado. Selecione o produto correto.'
    : 'Nenhum produto encontrado. Confira o nome ou SKU.'
}

function focusSearchShortcut(event) {
  const element = event.target
  const isEditing =
    element instanceof HTMLElement &&
    (element.isContentEditable || ['INPUT', 'SELECT', 'TEXTAREA'].includes(element.tagName))
  if (event.key === 'F2' && !isEditing) {
    event.preventDefault()
    searchInput.value?.focus()
  }
}

function focusClientShortcut(event) {
  const element = event.target
  const isEditing =
    element instanceof HTMLElement &&
    (element.isContentEditable || ['INPUT', 'SELECT', 'TEXTAREA'].includes(element.tagName))
  if (event.key === 'F1' && !isEditing) {
    event.preventDefault()
    document.getElementById(finalizeOpen.value ? 'final-client-search' : 'client-search')?.focus()
  }
}

function closeFinalizeShortcut(event) {
  if (event.key === 'Escape') {
    finalizeOpen.value = false
  }
}

function openFinalize() {
  finalMethod.value = 'dinheiro'
  finalPayments.value = [{ method: 'dinheiro', amount: Math.max(finalNetCents.value / 100, 0) }]
  finalDiscount.value = 0
  finalSurcharge.value = 0
  finalizeOpen.value = true
}

function addFinalizePayment() {
  finalPayments.value.push({
    method: finalMethod.value === 'prazo' ? 'pix' : 'dinheiro',
    amount: finalNetCents.value / 100,
  })
}

function removeFinalizePayment(index) {
  if (finalPayments.value.length <= 1) return
  finalPayments.value.splice(index, 1)
}

const finalizeDisabled = computed(() => {
  if (busy.value) return true
  if (finalIsCredit.value) {
    return !clients.value.length || !sale.value.clientId || !installmentSchedule.value.length
  }
  return finalPaidCents.value < finalNetCents.value
})

async function load() {
  try {
    await Promise.all([productsStore.load(), clientsStore.load(), sellersStore.load()])
    const productList = products.value.filter((product) => !product.is_demo)
    const sellerList = sellers.value
    if (!sale.value.sellerId && sellerList.length === 1) sale.value.sellerId = sellerList[0].id
    const first = productList[0]
    if (first && !sale.value.items[0].productId) {
      sale.value.items[0].productId = first.id
      sale.value.items[0].unitPrice = first.unit_price / 100
    }
  } catch (cause) {
    error.value = userMessage(cause)
  }
}

async function submitFinalize() {
  busy.value = true
  try {
    error.value = ''
    const payload = {
      client_id: sale.value.clientId ? Number(sale.value.clientId) : null,
      payment_method: finalMethod.value,
      seller_id: Number(sale.value.sellerId),
      is_credit: finalMethod.value === 'prazo',
      due_date: finalMethod.value === 'prazo' ? sale.value.dueDate : null,
      installments_count: finalMethod.value === 'prazo' ? Number(sale.value.installmentsCount) : 1,
      items: sale.value.items
        .filter((item) => item.productId)
        .map((item) => ({
          product_id: Number(item.productId),
          quantity: Number(item.quantity),
          unit_price: Math.round(Number(item.unitPrice) * 100),
        })),
      discount: Math.round(finalDiscount.value * 100),
      surcharge: Math.round(finalSurcharge.value * 100),
      payments:
        finalMethod.value === 'prazo'
          ? []
          : finalPayments.value.map((payment) => ({
              method: payment.method,
              amount: Math.round(Number(payment.amount || 0) * 100),
            })),
    }
    const createdSaleId = await invoke('create_sale', { sale: payload })
    const completedSale = {
      saleId: createdSaleId,
      date: new Date().toLocaleString('pt-BR'),
      seller: sellers.value.find((seller) => seller.id === Number(sale.value.sellerId))?.name ?? '',
      client: selectedClient.value?.name ?? null,
      paymentMethod: finalMethod.value,
      total: finalNetCents.value,
      items: sale.value.items
        .filter((item) => item.productId)
        .map((item) => ({
          name: selectedProduct(item.productId)?.name ?? '',
          gtin: selectedProduct(item.productId)?.gtin ?? '',
          quantity: Number(item.quantity),
          unitPrice: Math.round(Number(item.unitPrice) * 100),
        })),
      installments: installmentSchedule.value.map((item) => ({ ...item })),
    }
    finalizeOpen.value = false
    resetSale()
    receiptData.value = completedSale
    receiptConfirmOpen.value = true
  } catch (cause) {
    error.value = userMessage(cause)
  } finally {
    busy.value = false
  }
}

async function confirmPrint() {
  receiptConfirmOpen.value = false
  if (!receiptData.value) return
  const data = receiptData.value
  receiptData.value = data
  await nextTick()
  window.print()
  receiptData.value = null
}

function resetSale() {
  sale.value = {
    sellerId: sale.value.sellerId,
    clientId: '',
    dueDate: '',
    installmentsCount: 1,
    items: [{ productId: '', quantity: 1, unitPrice: 0 }],
  }
  finalMethod.value = 'dinheiro'
  finalPayments.value = [{ method: 'dinheiro', amount: Math.max(finalNetCents.value / 100, 0) }]
  finalDiscount.value = 0
  finalSurcharge.value = 0
}

function handleFinalizeMethod(method) {
  finalMethod.value = method
}

function handleFinalizeDiscount(value) {
  finalDiscount.value = value
}

function handleFinalizeSurcharge(value) {
  finalSurcharge.value = value
}

function handleFinalizeClientId(id) {
  sale.value.clientId = id
  clientSearch.value = ''
}

function handleFinalizeDueDate(date) {
  sale.value.dueDate = date
}

function handleFinalizeInstallmentsCount(count) {
  sale.value.installmentsCount = Number(count)
}

function handleFinalizePayment(index, amount) {
  finalPayments.value[index].amount = amount
}

function handleFinalizePaymentMethod(index, method) {
  finalPayments.value[index].method = method
}

onMounted(async () => {
  window.addEventListener('keydown', focusSearchShortcut)
  window.addEventListener('keydown', focusClientShortcut)
  window.addEventListener('keydown', closeFinalizeShortcut)
  await load()
  if (products.value.length) searchInput.value?.focus()
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', focusSearchShortcut)
  window.removeEventListener('keydown', focusClientShortcut)
  window.removeEventListener('keydown', closeFinalizeShortcut)
})
</script>

<template>
  <section class="narrow-content">
    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <div v-if="message" class="notice success" role="status">{{ message }}</div>
    <article class="card sale-card">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Operação</p>
          <h3>Registrar venda</h3>
        </div>
        <strong class="sale-total">{{ formatCurrency(finalNetCents) }}</strong>
      </div>
      <div>
        <div class="field-grid compact">
          <label
            >Forma de pagamento
            <select v-model="finalMethod">
              <option value="dinheiro">Dinheiro</option>
              <option value="cartao_credito">Cartão de crédito</option>
              <option value="cartao_debito">Cartão de débito</option>
              <option value="pix">PIX</option>
              <option value="prazo">Prazo</option>
            </select>
          </label>
          <label
            >Vendedor
            <select v-model="sale.sellerId" required>
              <option value="">Selecione</option>
              <option v-for="seller in sellers" :key="seller.id" :value="seller.id">
                {{ seller.name }}
              </option>
            </select>
          </label>
        </div>
        <button
          v-if="!sellers.length"
          class="link-button client-required-action"
          type="button"
          @click="emit('navigate', 'sellers')"
        >
          Cadastrar vendedor
        </button>
        <section v-if="finalIsCredit" class="credit-checkout">
          <h4>Pesquisa de Cliente</h4>
          <label for="client-search">Buscar pelo nome</label>
          <input
            id="client-search"
            v-model="clientSearch"
            type="search"
            autocomplete="off"
            placeholder="Digite o nome do cliente"
          />
          <ul v-if="filteredClients.length" class="product-results client-results">
            <li v-for="client in filteredClients" :key="client.id">
              <strong>{{ client.name }}</strong>
              <button class="link-button" type="button" @click="handleFinalizeClientId(client.id)">
                Selecionar
              </button>
            </li>
          </ul>
          <p v-if="selectedClient" class="selected-client">
            Cliente: <strong>{{ selectedClient.name }}</strong>
          </p>
          <div v-if="!clients.length" class="field-hint">
            Cadastre um cliente para habilitar a venda a prazo.
          </div>
          <button
            v-if="!clients.length"
            class="link-button"
            type="button"
            @click="emit('navigate', 'clients')"
          >
            Cadastrar cliente
          </button>
          <div class="field-grid compact installment-options">
            <label
              >Primeiro vencimento<input
                v-model="sale.dueDate"
                type="date"
                :min="new Date().toLocaleDateString('sv-SE')"
                required
            /></label>
            <label
              >Número de parcelas<input
                v-model.number="sale.installmentsCount"
                type="number"
                min="1"
                max="36"
                required
            /></label>
            <label
              >Valor a parcelar<strong class="installment-total">{{
                formatCurrency(finalNetCents)
              }}</strong></label
            >
          </div>
          <table v-if="installmentSchedule.length" class="installment-table">
            <thead>
              <tr>
                <th>Parcela</th>
                <th>Valor</th>
                <th>Vencimento</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="installment in installmentSchedule" :key="installment.number">
                <td>{{ installment.number }} / {{ installmentSchedule.length }}</td>
                <td>{{ formatCurrency(installment.amount) }}</td>
                <td>
                  {{ new Date(`${installment.dueDate}T00:00:00`).toLocaleDateString('pt-BR') }}
                </td>
              </tr>
            </tbody>
          </table>
        </section>
        <div class="product-search">
          <label for="sale-product-search">Adicionar produto</label>
          <div class="search-control">
            <input
              id="sale-product-search"
              ref="searchInput"
              v-model="productSearch"
              type="search"
              autocomplete="off"
              placeholder="Busque por nome ou SKU"
              aria-label="Buscar produto por nome ou SKU"
              @keydown.enter.prevent="addFromSearch"
            />
            <button
              class="secondary"
              type="button"
              :disabled="!productSearch.trim()"
              @click="addFromSearch"
            >
              Adicionar
            </button>
          </div>
          <ul
            v-if="filteredProducts.length"
            class="product-results"
            aria-label="Resultados da busca"
          >
            <li v-for="product in filteredProducts" :key="product.id">
              <span
                ><strong>{{ product.name }}</strong
                ><small
                  >GTIN {{ product.gtin || 'Não informado' }} · {{ product.stock_quantity }} un. ·
                  {{ formatCurrency(product.unit_price) }}</small
                ></span
              >
              <button
                class="link-button"
                type="button"
                :disabled="product.stock_quantity < 1"
                @click="addProduct(product)"
              >
                {{ product.stock_quantity > 0 ? 'Adicionar' : 'Sem estoque' }}
              </button>
            </li>
          </ul>
        </div>
        <div v-if="sale.items.some((item) => item.productId)" class="sale-items">
          <h4>Itens da venda</h4>
          <div
            v-for="item in sale.items.filter((line) => line.productId)"
            :key="item.productId"
            class="sale-line"
          >
            <span class="sale-product-name">{{ selectedProduct(item.productId)?.name }}</span>
            <input
              v-model.number="item.quantity"
              type="number"
              min="1"
              :max="selectedProduct(item.productId)?.stock_quantity"
              required
              :aria-label="`Quantidade de ${selectedProduct(item.productId)?.name}`"
            />
            <input
              v-model.number="item.unitPrice"
              type="number"
              min="0"
              step="0.01"
              required
              :aria-label="`Preço unitário de ${selectedProduct(item.productId)?.name}`"
            />
            <strong class="line-subtotal">{{
              formatCurrency(Math.round(Number(item.quantity) * Number(item.unitPrice) * 100))
            }}</strong>
            <button
              class="icon-button"
              type="button"
              :aria-label="`Remover ${selectedProduct(item.productId)?.name}`"
              @click="sale.items.splice(sale.items.indexOf(item), 1)"
            >
              ×
            </button>
          </div>
        </div>
        <div class="sale-actions">
          <span>{{ sale.items.filter((item) => item.productId).length }} produtos no carrinho</span>
        </div>
        <p v-if="!products.length" class="empty-state">
          Cadastre um produto real e registre estoque para iniciar uma venda.
        </p>
        <button
          class="primary"
          type="button"
          :disabled="
            busy ||
            !products.length ||
            !sellers.length ||
            !sale.sellerId ||
            !sale.items.some((item) => item.productId)
          "
          @click="openFinalize"
        >
          {{ busy ? 'Registrando...' : 'Finalizar venda' }}
        </button>
      </div>
    </article>

    <SaleFinalizeDialog
      :open="finalizeOpen"
      :method="finalMethod"
      :payments="finalPayments"
      :discount="finalDiscount"
      :surcharge="finalSurcharge"
      :netCents="finalNetCents"
      :paidCents="finalPaidCents"
      :changeCents="finalChangeCents"
      :remainingCents="finalRemainingCents"
      :disabled="finalizeDisabled"
      :busy="busy"
      :isCredit="finalIsCredit"
      :installments="installmentSchedule"
      :clients="clients"
      :clientId="sale.clientId"
      :dueDate="sale.dueDate"
      :installmentsCount="sale.installmentsCount"
      @submit="submitFinalize"
      @cancel="finalizeOpen = false"
      @update:method="handleFinalizeMethod"
      @update:discount="handleFinalizeDiscount"
      @update:surcharge="handleFinalizeSurcharge"
      @add-payment="addFinalizePayment"
      @remove-payment="removeFinalizePayment"
      @update:clientId="handleFinalizeClientId"
      @update:dueDate="handleFinalizeDueDate"
      @update:installmentsCount="handleFinalizeInstallmentsCount"
      @update:payment="handleFinalizePayment"
      @update:payment-method="handleFinalizePaymentMethod"
    />

    <div v-if="finalizeOpen && busy" class="dav-modal-backdrop">
      <div class="dav-await">
        <div class="dav-spinner"></div>
        <p>Aguarde</p>
        <strong>Finalizando DAV...</strong>
      </div>
    </div>

    <div v-if="receiptConfirmOpen" class="dav-modal-backdrop">
      <div
        class="dav-window dav-window--small"
        role="dialog"
        aria-modal="true"
        aria-label="Confirmação de impressão"
      >
        <div class="dav-window-header">
          <span>Confirma Impressão?</span>
          <button
            class="dav-window-close"
            type="button"
            aria-label="Fechar diálogo"
            @click="receiptConfirmOpen = false"
          >
            ×
          </button>
        </div>
        <div class="confirm-body">
          <div class="dav-question-icon">?</div>
          <p>Deseja imprimir o comprovante?</p>
        </div>
        <div class="dav-window-footer centered">
          <div class="dav-actions">
            <button class="secondary" type="button" @click="receiptConfirmOpen = false">Não</button>
            <button class="primary" type="button" @click="confirmPrint">Sim</button>
          </div>
        </div>
      </div>
    </div>

    <SaleReceipt v-if="receiptData" :receipt="receiptData" />
  </section>
</template>
