<script setup>
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import MetricCard from './MetricCard.vue'
import RevenueChart from './RevenueChart.vue'
import ExpirationChart from './ExpirationChart.vue'
import { userMessage } from '../../lib/userMessage'
import { formatCurrency } from '../../lib/format'

const emit = defineEmits(['navigate'])

const dashboard = ref({
  faturamento: 0,
  lucro: 0,
  saldo_caixa: 0,
  estoque_total: 0,
  contas_receber: 0,
  vendas_mes: 0,
  vendas_total: 0,
})
const revenue = ref([])
const expiring = ref([])
const receivables = ref([])
const products = ref([])
const clients = ref([])
const cashStatus = ref({ is_open: false })
const error = ref('')
const loading = ref(true)
const checklistDismissed = ref(localStorage.getItem('setupChecklistDismissed') === 'true')
const realProducts = computed(() => products.value.filter((product) => !product.is_demo))
const realClients = computed(() => clients.value.filter((client) => !client.is_demo))
const setupTasks = computed(() => [
  {
    id: 'client',
    label: 'Cadastre um cliente',
    complete: realClients.value.length > 0,
    view: 'clients',
  },
  {
    id: 'product',
    label: 'Cadastre um produto',
    complete: realProducts.value.length > 0,
    view: 'inventory',
  },
  {
    id: 'stock',
    label: 'Registre uma entrada de estoque',
    complete: realProducts.value.some((product) => product.stock_quantity > 0),
    view: 'inventory',
  },
  { id: 'cash', label: 'Abra o caixa do dia', complete: cashStatus.value.is_open, view: 'cash' },
  {
    id: 'sale',
    label: 'Registre a primeira venda',
    complete: dashboard.value.vendas_total > 0,
    view: 'sales',
  },
])
const pendingTasks = computed(() => setupTasks.value.filter((task) => !task.complete))
const hasDemoRecords = computed(
  () =>
    products.value.some((product) => product.is_demo) ||
    clients.value.some((client) => client.is_demo),
)
const overdueReceivables = computed(() =>
  receivables.value.filter(
    (item) => item.due_date && item.due_date < new Date().toISOString().slice(0, 10),
  ),
)

async function load() {
  try {
    error.value = ''
    const [
      summary,
      monthly,
      productList,
      clientList,
      currentCash,
      openReceivables,
      expiringProducts,
    ] = await Promise.all([
      invoke('get_dashboard_data'),
      invoke('get_monthly_revenue'),
      invoke('list_products'),
      invoke('list_clients'),
      invoke('get_cash_status'),
      invoke('list_receivables'),
      invoke('list_expiring_products'),
    ])
    dashboard.value = summary
    revenue.value = monthly
    expiring.value = expiringProducts
    products.value = productList
    clients.value = clientList
    cashStatus.value = currentCash
    receivables.value = openReceivables
  } catch (cause) {
    error.value = userMessage(cause)
  } finally {
    loading.value = false
  }
}

function dismissChecklist() {
  checklistDismissed.value = true
  localStorage.setItem('setupChecklistDismissed', 'true')
}

onMounted(load)
</script>

<template>
  <section>
    <div v-if="loading" class="loading-state" role="status">Carregando resumo e alertas...</div>
    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <article v-if="!loading && pendingTasks.length && !checklistDismissed" class="card setup-card">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Configuração inicial</p>
          <h3>Comece por aqui</h3>
        </div>
        <button class="text-button" type="button" @click="dismissChecklist">Dispensar</button>
      </div>
      <ol class="setup-list">
        <li v-for="task in setupTasks" :key="task.id" :class="{ complete: task.complete }">
          <span class="setup-check" :aria-label="task.complete ? 'Concluído' : 'Pendente'">{{
            task.complete ? '✓' : ''
          }}</span>
          <span>{{ task.label }}</span>
          <button
            v-if="!task.complete"
            class="link-button"
            type="button"
            @click="emit('navigate', task.view)"
          >
            Abrir
          </button>
          <small v-else>Concluído</small>
        </li>
      </ol>
    </article>
    <p v-if="hasDemoRecords" class="notice info" role="status">
      Há registros de demonstração identificados. Eles não contam como etapas reais de configuração.
    </p>
    <div
      v-if="!loading && !cashStatus.is_open"
      class="notice warning dashboard-alert"
      role="status"
    >
      <span
        ><strong>Caixa fechado.</strong> Abra o caixa antes de fazer vendas à vista ou receber
        pagamentos.</span
      >
      <button class="link-button" type="button" @click="emit('navigate', 'cash')">
        Abrir caixa
      </button>
    </div>
    <div
      v-if="!loading && overdueReceivables.length"
      class="notice error dashboard-alert"
      role="status"
    >
      <span
        ><strong>{{ overdueReceivables.length }} conta(s) vencida(s).</strong> Confira os
        recebimentos pendentes.</span
      >
      <button class="link-button" type="button" @click="emit('navigate', 'receivables')">
        Ver recebíveis
      </button>
    </div>
    <div
      v-if="!loading && !error && dashboard.vendas_total === 0"
      class="empty-state welcome-state"
    >
      <span>Ainda não há vendas reais registradas.</span>
      <button class="link-button" type="button" @click="emit('navigate', 'sales')">
        Abrir vendas
      </button>
    </div>
    <div class="stats-grid">
      <MetricCard label="Faturamento" :value="formatCurrency(dashboard.faturamento)" tone="blue" />
      <MetricCard label="Lucro bruto" :value="formatCurrency(dashboard.lucro)" tone="green" />
      <MetricCard
        label="Saldo de caixa"
        :value="formatCurrency(dashboard.saldo_caixa)"
        tone="gold"
      />
      <MetricCard
        label="Contas a receber"
        :value="formatCurrency(dashboard.contas_receber)"
        tone="red"
      />
    </div>

    <div class="charts-grid finance-chart-grid">
      <article class="card chart-card finance-chart-card">
        <div class="section-heading">
          <h3>Faturamento por mês</h3>
          <span>{{ dashboard.vendas_mes }} vendas neste mês</span>
        </div>
        <RevenueChart :data="revenue" />
      </article>
      <article class="card chart-card finance-chart-card">
        <div class="section-heading"><h3>Produtos próximos do vencimento</h3></div>
        <ExpirationChart :data="expiring" />
      </article>
    </div>
  </section>
</template>
