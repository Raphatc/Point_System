<script setup>
import { onMounted, ref } from 'vue'
import { formatCurrency } from '../../lib/format'
import { useProductsStore } from '../../stores/products'

const productsStore = useProductsStore()
const products = productsStore.products
const error = productsStore.error
const message = productsStore.message
const loading = productsStore.loading
const product = ref({
  name: '',
  sku: '',
  gtin: '',
  category: 'Mercearia',
  unitPrice: 0,
  costPrice: 0,
})
const stock = ref({ productId: '', quantity: 1, costPrice: 0, expirationDate: '', supplier: '' })

async function loadProducts() {
  await productsStore.load()
  if (!stock.value.productId) {
    const realProduct = productsStore.products.find((item) => !item.is_demo)
    if (realProduct) stock.value.productId = realProduct.id
  }
}

async function createProduct() {
  const created = await productsStore.createProduct({
    name: product.value.name.trim(),
    sku: product.value.sku.trim(),
    gtin: product.value.gtin.trim(),
    category: product.value.category,
    unit_price: Math.round(Number(product.value.unitPrice) * 100),
    cost_price: Math.round(Number(product.value.costPrice) * 100),
  })
  if (created) {
    product.value = {
      name: '',
      sku: '',
      gtin: '',
      category: 'Mercearia',
      unitPrice: 0,
      costPrice: 0,
    }
  }
}

async function addStock() {
  const created = await productsStore.addStock({
    product_id: Number(stock.value.productId),
    quantity: Number(stock.value.quantity),
    cost_price: Math.round(Number(stock.value.costPrice) * 100),
    expiration_date: stock.value.expirationDate,
    supplier: stock.value.supplier.trim(),
  })
  if (created) {
    stock.value = {
      productId: stock.value.productId,
      quantity: 1,
      costPrice: 0,
      expirationDate: '',
      supplier: '',
    }
  }
}

onMounted(loadProducts)
</script>

<template>
  <section>
    <div v-if="loading" class="loading-state" role="status">Carregando produtos...</div>
    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <div v-if="message" class="notice success">{{ message }}</div>
    <div class="forms-grid">
      <article class="card">
        <h3>Novo produto</h3>
        <form class="field-grid" @submit.prevent="createProduct">
          <label>Nome do produto<input v-model="product.name" required maxlength="120" /></label>
          <label
            >SKU ou código de barras<input v-model="product.sku" required maxlength="80"
          /></label>
          <label
            >GTIN (opcional)<input v-model="product.gtin" inputmode="numeric" maxlength="14"
          /></label>
          <label>Categoria<input v-model="product.category" required maxlength="80" /></label>
          <label
            >Preço de venda (R$)<input
              v-model.number="product.unitPrice"
              type="number"
              min="0"
              step="0.01"
              required
          /></label>
          <label
            >Custo unitário (R$)<input
              v-model.number="product.costPrice"
              type="number"
              min="0"
              step="0.01"
              required
          /></label>
          <button class="primary" type="submit">Salvar produto</button>
        </form>
      </article>
      <article class="card">
        <h3>Entrada de estoque</h3>
        <form class="field-grid" @submit.prevent="addStock">
          <select v-model="stock.productId" required>
            <option value="">Selecione o produto</option>
            <option v-for="item in products" :key="item.id" :value="item.id">
              {{ item.name }}
            </option>
          </select>
          <label
            >Quantidade de unidades<input
              v-model.number="stock.quantity"
              type="number"
              min="1"
              required
          /></label>
          <label
            >Custo unitário (R$)<input
              v-model.number="stock.costPrice"
              type="number"
              min="0"
              step="0.01"
              required
          /></label>
          <label>Validade (opcional)<input v-model="stock.expirationDate" type="date" /></label>
          <label>Fornecedor (opcional)<input v-model="stock.supplier" maxlength="120" /></label>
          <button class="primary" type="submit" :disabled="!products.length">
            Registrar entrada
          </button>
        </form>
      </article>
    </div>
    <article class="card table-card">
      <div class="section-heading">
        <h3>Estoque atual</h3>
        <span>{{ products.length }} produtos</span>
      </div>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Produto</th>
              <th>GTIN</th>
              <th>Categoria</th>
              <th>Quantidade</th>
              <th>Preço unitário</th>
              <th>Custo médio</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in products" :key="item.id">
              <td>
                {{ item.name }} <span v-if="item.is_demo" class="demo-badge">Demonstração</span>
              </td>
              <td>{{ item.gtin || '—' }}</td>
              <td>{{ item.category }}</td>
              <td>{{ item.stock_quantity }}</td>
              <td>{{ formatCurrency(item.unit_price) }}</td>
              <td>{{ formatCurrency(item.cost_average) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p v-if="!products.length" class="empty-state">Nenhum produto cadastrado.</p>
    </article>
  </section>
</template>
