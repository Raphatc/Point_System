<script setup>
import ClientsPanel from './ClientsPanel.vue'
import InventoryPanel from './InventoryPanel.vue'
import SellersPanel from './SellersPanel.vue'

defineProps({ selectedCatalog: { type: String, default: 'products' } })
const emit = defineEmits(['update:selectedCatalog'])
</script>

<template>
  <section>
    <div class="catalog-tabs" role="tablist" aria-label="Cadastros auxiliares">
      <button
        class="catalog-tab"
        :class="{ active: selectedCatalog === 'products' }"
        role="tab"
        :aria-selected="selectedCatalog === 'products'"
        @click="emit('update:selectedCatalog', 'products')"
      >
        Produtos e estoque
      </button>
      <button
        class="catalog-tab"
        :class="{ active: selectedCatalog === 'clients' }"
        role="tab"
        :aria-selected="selectedCatalog === 'clients'"
        @click="emit('update:selectedCatalog', 'clients')"
      >
        Clientes
      </button>
      <button
        class="catalog-tab"
        :class="{ active: selectedCatalog === 'sellers' }"
        role="tab"
        :aria-selected="selectedCatalog === 'sellers'"
        @click="emit('update:selectedCatalog', 'sellers')"
      >
        Vendedores
      </button>
    </div>
    <InventoryPanel v-if="selectedCatalog === 'products'" />
    <ClientsPanel v-else />
    <SellersPanel v-if="selectedCatalog === 'sellers'" />
  </section>
</template>
