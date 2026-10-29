<script setup>
import { onMounted, ref } from 'vue'
import { useSellersStore } from '../../stores/sellers'

const sellersStore = useSellersStore()
const sellers = sellersStore.sellers
const error = sellersStore.error
const message = sellersStore.message
const name = ref('')

async function load() {
  await sellersStore.load()
}

async function create() {
  const created = await sellersStore.createSeller({ name: name.value.trim() })
  if (created) {
    name.value = ''
    message.value = 'Vendedor cadastrado.'
  }
}

onMounted(load)
</script>

<template>
  <section>
    <div v-if="error" class="notice error" role="alert">{{ error }}</div>
    <div v-if="message" class="notice success" role="status">{{ message }}</div>
    <article class="card">
      <h3>Novo vendedor</h3>
      <form class="seller-form" @submit.prevent="create">
        <label>Nome<input v-model="name" required maxlength="100" /></label>
        <button class="primary" type="submit">Cadastrar vendedor</button>
      </form>
    </article>
    <article class="card table-card">
      <div class="section-heading">
        <h3>Vendedores ativos</h3>
        <span>{{ sellers.length }} cadastrados</span>
      </div>
      <ul v-if="sellers.length" class="list-box">
        <li v-for="seller in sellers" :key="seller.id">
          <strong>{{ seller.name }}</strong
          ><span class="status-pill open">Ativo</span>
        </li>
      </ul>
      <p v-else class="empty-state">Cadastre um vendedor para habilitar o registro de vendas.</p>
    </article>
  </section>
</template>
