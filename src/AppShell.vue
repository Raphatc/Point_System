<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { formatCurrency } from './lib/format'
import { useClientsStore } from './stores/clients'
import { useProductsStore } from './stores/products'
import CashModule from './components/cash/CashModule.vue'
import FinanceModule from './components/finance/FinanceModule.vue'
import SupportCatalog from './components/catalog/SupportCatalog.vue'
import DashboardPanel from './components/finance/DashboardPanel.vue'
import BackupPanel from './components/backup/BackupPanel.vue'

const activeView = ref('cash')
const supportCatalog = ref('products')
const menuOpen = ref(false)
const productSearch = ref('')
const clientSearch = ref('')

const version = ref('2026')
const operator = ref('Operador')
const buildStamp = __APP_BUILD__

const navigation = [
  { id: 'dashboard', label: 'Painel', mark: '00', view: 'dashboard' },
  { id: 'cash', label: 'Caixa / Vendas', mark: '01', view: 'cash' },
]

const supportNavigation = [
  { id: 'inventory', label: 'Produtos', mark: '02', view: 'support', catalog: 'products' },
  { id: 'clients', label: 'Clientes', mark: '03', view: 'support', catalog: 'clients' },
  { id: 'sellers', label: 'Vendedores', mark: '04', view: 'support', catalog: 'sellers' },
]

const financeNavigation = [{ id: 'receivables', label: 'Recebíveis', mark: '05', view: 'finance' }]

const backupNavigation = [{ id: 'backup', label: 'Backup', mark: '06', view: 'backup' }]

const menuItems = [...navigation, ...supportNavigation, ...financeNavigation, ...backupNavigation]

const topbar = computed(() => {
  switch (activeView.value) {
    case 'cash':
      return { eyebrow: 'Operação', label: 'Caixa / Vendas' }
    case 'finance':
      return { eyebrow: 'Operação', label: 'Financeiro' }
    case 'support':
      return { eyebrow: 'Cadastros', label: 'Cadastros auxiliares' }
    case 'backup':
      return { eyebrow: 'Dados', label: 'Backup do banco' }
    default:
      return { eyebrow: 'Operação', label: 'Painel' }
  }
})

const now = ref(new Intl.DateTimeFormat('pt-BR', { dateStyle: 'full' }).format(new Date()))

function formatNow() {
  return new Intl.DateTimeFormat('pt-BR', { dateStyle: 'full' }).format(new Date())
}

const productsStore = useProductsStore()
const clientsStore = useClientsStore()

const filteredProducts = computed(() => {
  const term = productSearch.value.trim().toLocaleLowerCase('pt-BR')
  if (!term) return []
  return productsStore.products.filter(
    (product) =>
      product.name.toLocaleLowerCase('pt-BR').includes(term) ||
      product.sku.toLocaleLowerCase('pt-BR').includes(term) ||
      product.gtin?.toLocaleLowerCase('pt-BR').includes(term),
  )
})

const filteredClients = computed(() => {
  const term = clientSearch.value.trim().toLocaleLowerCase('pt-BR')
  if (!term) return []
  return clientsStore.clients.filter((client) =>
    client.name.toLocaleLowerCase('pt-BR').includes(term),
  )
})

function isMenuItemActive(item) {
  if (item.view === 'support') {
    return activeView.value === 'support' && supportCatalog.value === (item.catalog ?? 'products')
  }
  return activeView.value === item.view
}

function navigateTo(item) {
  if (item.view === 'support') {
    supportCatalog.value = item.catalog ?? 'products'
    activeView.value = 'support'
  } else {
    activeView.value = item.view
  }
  menuOpen.value = false
}

function handleNavigation(target) {
  if (target === 'dashboard') {
    activeView.value = 'dashboard'
    return
  }
  if (target === 'cash' || target === 'sales') {
    activeView.value = 'cash'
    return
  }
  if (target === 'finance' || target === 'receivables') {
    activeView.value = 'finance'
    return
  }
  if (target === 'clients') {
    supportCatalog.value = 'clients'
    activeView.value = 'support'
    return
  }
  if (target === 'sellers') {
    supportCatalog.value = 'sellers'
    activeView.value = 'support'
    return
  }
  if (target === 'inventory') {
    supportCatalog.value = 'products'
    activeView.value = 'support'
  }
}

function toggleMenu() {
  menuOpen.value = !menuOpen.value
}

function onGlobalKeydown(event) {
  if (event.key === 'F11') {
    event.preventDefault()
    toggleMenu()
    return
  }
  if (event.key === 'Escape') {
    menuOpen.value = false
  }
}

onMounted(() => {
  productsStore.load()
  clientsStore.load()
  now.value = formatNow()
  window.addEventListener('keydown', onGlobalKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onGlobalKeydown)
})
</script>

<template>
  <div class="app-shell">
    <header class="dav-header">
      <div class="dav-header-left">
        <button
          class="dav-menu-toggle"
          type="button"
          :aria-expanded="menuOpen"
          :aria-controls="'dav-menu-panel'"
          :aria-label="menuOpen ? 'Fechar menu de navegação' : 'Abrir menu de navegação'"
          @click="menuOpen = !menuOpen"
        >
          ☰
        </button>
        <div class="dav-logo" aria-hidden="true">DAV</div>
        <p class="dav-slogan">Distribuidora Point</p>
      </div>
      <div class="dav-meta">
        <span>Versão: {{ version }}</span>
        <span>Build: {{ buildStamp }}</span>
        <span>Operador: {{ operator }}</span>
      </div>
    </header>

    <div class="dav-searchbar">
      <span class="dav-active-module">{{ topbar.label }}</span>
      <div class="dav-search-box">
        <input
          id="dav-product-search"
          v-model="productSearch"
          type="search"
          autocomplete="off"
          placeholder="Buscar produto por nome ou SKU"
          aria-label="Buscar produto por nome ou SKU"
        />
        <span class="dav-search-glyph" aria-hidden="true"></span>
        <ul v-if="filteredProducts.length" class="dav-search-results">
          <li
            v-for="product in filteredProducts"
            :key="product.id"
            @click="productSearch = product.name"
          >
            <span
              ><strong>{{ product.name }}</strong
              ><small
                >GTIN {{ product.gtin || 'Não informado' }} · {{ product.stock_quantity }} un. ·
                {{ formatCurrency(product.unit_price) }}</small
              ></span
            >
            <span class="dav-result-type">Produto</span>
          </li>
        </ul>
        <p v-else-if="productSearch.trim()" class="dav-search-empty">
          Nenhum produto encontrado para "{{ productSearch }}".
        </p>
      </div>
      <div class="dav-search-box">
        <input
          id="dav-client-search"
          v-model="clientSearch"
          type="search"
          autocomplete="off"
          placeholder="Buscar cliente por nome"
          aria-label="Buscar cliente por nome"
        />
        <span class="dav-search-glyph" aria-hidden="true"></span>
        <ul v-if="filteredClients.length" class="dav-search-results">
          <li
            v-for="client in filteredClients"
            :key="client.id"
            @click="clientSearch = client.name"
          >
            <span
              ><strong>{{ client.name }}</strong
              ><small>Cliente</small></span
            >
            <span class="dav-result-type">Cliente</span>
          </li>
        </ul>
        <p v-else-if="clientSearch.trim()" class="dav-search-empty">
          Nenhum cliente encontrado para "{{ clientSearch }}".
        </p>
      </div>
    </div>

    <div v-if="menuOpen" class="dav-menu-backdrop" @click="menuOpen = false"></div>
    <div v-if="menuOpen" class="dav-menu-panel" id="dav-menu-panel">
      <div class="dav-window-header">
        <span>Menu</span>
        <button
          class="dav-window-close"
          type="button"
          aria-label="Fechar menu"
          @click="menuOpen = false"
        >
          ×
        </button>
      </div>
      <ul class="dav-menu-list">
        <li v-for="item in menuItems" :key="item.id">
          <button
            type="button"
            class="dav-menu-item"
            :class="{ active: isMenuItemActive(item) }"
            :aria-current="isMenuItemActive(item) ? 'page' : undefined"
            @click="navigateTo(item)"
          >
            <span class="dav-menu-mark">{{ item.mark }}</span>
            <span>{{ item.label }}</span>
          </button>
        </li>
      </ul>
      <div class="dav-menu-footer">
        <span>Banco local conectado</span>
      </div>
    </div>

    <main class="content">
      <header class="topbar">
        <div>
          <p class="eyebrow">{{ topbar.eyebrow }}</p>
          <h1>{{ topbar.label }}</h1>
        </div>
        <time>{{ now }}</time>
      </header>

      <Transition name="view" mode="out-in">
        <CashModule v-if="activeView === 'cash'" key="cash" @navigate="handleNavigation" />
        <FinanceModule
          v-else-if="activeView === 'finance'"
          key="finance"
          @navigate="handleNavigation"
        />
        <SupportCatalog
          v-else-if="activeView === 'support'"
          key="support"
          v-model:selected-catalog="supportCatalog"
        />
        <BackupPanel
          v-else-if="activeView === 'backup'"
          key="backup"
          @navigate="handleNavigation"
        />
        <DashboardPanel v-else key="dashboard" @navigate="handleNavigation" />
      </Transition>
    </main>
  </div>
</template>
